/**
 * Background-side proxy for fetch() — every popup / sidepanel request is
 * proxied here to avoid CORS, attach Idempotency-Key + trace_id, and apply
 * redaction before errors surface.
 * Audit ref: M-19.
 */
import { getAccessToken, getConfig } from '../lib/storage';
import { redactAccessTokens } from '../lib/redaction';

export interface ApiRequest {
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  path: string;
  body?: unknown;
  trace_id?: string;
  idempotency_key?: string;
}

export interface ApiResponse {
  ok: boolean;
  status: number;
  data?: unknown;
  error?: string;
  trace_id: string;
  idempotency_key?: string;
}

export async function proxyApiRequest(req: ApiRequest): Promise<ApiResponse> {
  const cfg = await getConfig();
  const token = await getAccessToken();
  const traceId = req.trace_id ?? crypto.randomUUID();
  const idempotencyKey = req.idempotency_key ?? (req.method === 'GET' ? undefined : crypto.randomUUID());

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'x-trace-id': traceId,
  };
  if (idempotencyKey) headers['Idempotency-Key'] = idempotencyKey;
  if (token) headers.Authorization = `Bearer ${token}`;

  const url = new URL(req.path, cfg.apiBase).toString();

  try {
    const res = await fetch(url, {
      method: req.method,
      headers,
      body: req.body !== undefined ? JSON.stringify(req.body) : undefined,
    });
    const text = await res.text();
    const data = text ? safeJson(text) : null;
    return {
      ok: res.ok,
      status: res.status,
      trace_id: traceId,
      idempotency_key: idempotencyKey,
      ...(data !== null ? { data } : {}),
      ...(res.ok ? {} : { error: redactAccessTokens(text).slice(0, 250) }),
    };
  } catch (err) {
    return {
      ok: false,
      status: 0,
      trace_id: traceId,
      error: redactAccessTokens(err instanceof Error ? err.message : String(err)),
    };
  }
}

function safeJson(s: string): unknown {
  try { return JSON.parse(s); } catch { return null; }
}
