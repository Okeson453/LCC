import { getAccessToken, getConfig } from './storage';
import { redactAccessTokens } from './redaction';
import { dedupe, buildKey } from './dedupe';

export interface FetchInput {
  method: 'GET' | 'POST';
  path: string;
  body?: unknown;
  traceId?: string;
  idempotencyKey?: string;
}

export interface FetchOutput<T = unknown> {
  ok: boolean;
  status: number;
  data?: T;
  error?: string;
}

export async function apiFetch<T>({ method, path, body, traceId, idempotencyKey }: FetchInput): Promise<FetchOutput<T>> {
  const key = buildKey(method, path);
  // Read-only calls are deduped while in-flight to coalesce concurrent reloads.
  if (method === 'GET') {
    return dedupe(key, () => doFetch<T>({ method, path, body, traceId, idempotencyKey }));
  }
  return doFetch<T>({ method, path, body, traceId, idempotencyKey });
}

async function doFetch<T>({ method, path, body, traceId, idempotencyKey }: FetchInput): Promise<FetchOutput<T>> {
  const cfg = await getConfig();
  const token = await getAccessToken();
  const url = new URL(path, cfg.apiBase).toString();
  const tid = traceId ?? crypto.randomUUID();
  const ik = idempotencyKey ?? (method === 'POST' ? crypto.randomUUID() : undefined);

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'x-trace-id': tid,
  };
  if (token) headers.Authorization = `Bearer ${token}`;
  if (ik) headers['Idempotency-Key'] = ik;

  try {
    const res = await fetch(url, {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    const text = await res.text();
    const json = text ? safeParse(text) : null;

    return {
      ok: res.ok,
      status: res.status,
      ...(json !== null ? { data: json as T } : {}),
      ...(res.ok ? {} : { error: redactAccessTokens(text).slice(0, 200) }),
    };
  } catch (err) {
    return { ok: false, status: 0, error: redactAccessTokens(err instanceof Error ? err.message : String(err)) };
  }
}

function safeParse(text: string): unknown {
  try { return JSON.parse(text); } catch { return null; }
}
