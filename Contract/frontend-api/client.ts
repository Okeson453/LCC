/**
 * Base API client — fetch wrapper.
 *
 * - Injects `Authorization: Bearer <token>` from NextAuth session.
 * - Injects `x-trace-id` and (for mutations) `Idempotency-Key`.
 * - Returns parsed JSON or throws `ApiError` with classification.
 * - Validates response against zod schema when provided.
 */

import { API_BASE, API_TIMEOUT_MS } from './config';
import { ApiError, classifyError, isRetryable } from './errors';
import { buildAuthHeaders, freshIdempotencyKey } from './auth-headers';

export type HttpMethod = 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';

export interface ApiClientConfig {
  /** Bearer token. If null, no Authorization header is sent. */
  getToken: () => string | null;
  /** Called on 401 — typically redirects to /auth/linkedin/start. */
  onUnauthorized?: () => void;
  /** Called when restricted-state is detected. */
  onRestricted?: () => void;
  /** Called on app version mismatch (backend says client is stale). */
  onVersionMismatch?: () => void;
}

export interface ApiFetchOptions<TBody = unknown> {
  method?: HttpMethod;
  body?: TBody;
  /** Query parameters (will be URL-encoded). */
  query?: Record<string, string | number | boolean | undefined>;
  /** Optional zod schema for response validation. */
  schema?: import('zod').ZodType<unknown>;
  /** Per-call timeout override. */
  timeoutMs?: number;
  /** Skip auth header. */
  skipAuth?: boolean;
  /** Pass a specific idempotency key (mutations). */
  idempotencyKey?: string;
  /** Skip Idempotency-Key generation (for GETs). */
  skipIdempotency?: boolean;
  /** Custom headers. */
  headers?: Record<string, string>;
  /** Allow a GET to be retried once on failure. */
  retry?: boolean;
}

let globalConfig: ApiClientConfig | null = null;

export function configureApiClient(config: ApiClientConfig): void {
  globalConfig = config;
}

export async function apiFetch<TResponse = unknown>(
  path: string,
  options: ApiFetchOptions = {},
): Promise<TResponse> {
  const config = globalConfig;
  if (!config) {
    throw new ApiError({
      kind: 'unknown',
      message: 'API client not configured',
      status: 0,
      traceId: null,
    });
  }

  const method = options.method ?? 'GET';
  const url = buildUrl(path, options.query);

  const headers: Record<string, string> = {
    Accept: 'application/json',
    ...(options.body ? { 'Content-Type': 'application/json' } : {}),
    ...buildAuthHeaders(
      options.skipAuth ? null : config.getToken(),
      {
        idempotencyKey:
          method !== 'GET' && !options.skipIdempotency
            ? options.idempotencyKey ?? freshIdempotencyKey()
            : undefined,
      },
    ),
    ...(options.headers ?? {}),
  };

  const timeoutMs = options.timeoutMs ?? API_TIMEOUT_MS;
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  let response: Response;
  try {
    response = await fetch(url, {
      method,
      headers,
      body: options.body ? JSON.stringify(options.body) : undefined,
      credentials: 'include',
      signal: controller.signal,
    });
  } catch (err) {
    clearTimeout(timeoutId);
    const classified = classifyError(err);
    if (isRetryable(classified) && (options.retry ?? method === 'GET')) {
      return apiFetch(path, { ...options, retry: false });
    }
    throw classified;
  }
  clearTimeout(timeoutId);

  const traceId = response.headers.get('x-trace-id');

  if (!response.ok) {
    const error = await parseErrorBody(response);
    const apiErr = new ApiError({
      kind: classifyStatus(response.status, error),
      message: error?.error?.message ?? response.statusText,
      status: response.status,
      traceId,
      details: error?.error?.details ?? error,
    });

    if (apiErr.kind === 'unauthorized') config.onUnauthorized?.();
    if (apiErr.kind === 'restricted') config.onRestricted?.();
    if (apiErr.kind === 'version_mismatch') config.onVersionMismatch?.();

    if (isRetryable(apiErr) && (options.retry ?? method === 'GET')) {
      return apiFetch(path, { ...options, retry: false });
    }

    throw apiErr;
  }

  const text = await response.text();
  if (!text) return undefined as TResponse;

  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new ApiError({
      kind: 'unknown',
      message: 'Failed to parse JSON response',
      status: response.status,
      traceId,
    });
  }

  if (options.schema) {
    const result = options.schema.safeParse(json);
    if (!result.success) {
      throw new ApiError({
        kind: 'validation',
        message: 'API contract violation: ' + result.error.issues[0]?.message,
        status: response.status,
        traceId,
        details: result.error.issues,
      });
    }
    return result.data as TResponse;
  }

  return json as TResponse;
}

function buildUrl(path: string, query?: Record<string, string | number | boolean | undefined>): string {
  const base = API_BASE + path;
  if (!query) return base;
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) {
    if (v != null) params.append(k, String(v));
  }
  const qs = params.toString();
  return qs ? `${base}?${qs}` : base;
}

async function parseErrorBody(response: Response): Promise<{ error?: { message?: string; details?: unknown } } | null> {
  try {
    const text = await response.text();
    if (!text) return null;
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function classifyStatus(
  status: number,
  body: { error?: { code?: string } } | null,
): import('./errors').ApiErrorKind {
  if (body?.error?.code === 'GOVERNANCE_DENIED') return 'governance_denied';
  if (body?.error?.code === 'VERSION_MISMATCH') return 'version_mismatch';
  switch (status) {
    case 400:
    case 422:
      return 'validation';
    case 401:
      return 'unauthorized';
    case 403:
      return 'forbidden';
    case 404:
      return 'not_found';
    case 409:
      return 'restricted';
    case 429:
      return 'rate_limited';
    case 500:
    case 502:
    case 503:
    case 504:
      return 'server_error';
    default:
      return 'unknown';
  }
}
