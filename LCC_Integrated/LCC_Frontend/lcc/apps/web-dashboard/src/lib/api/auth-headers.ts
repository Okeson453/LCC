/**
 * Header helpers — Bearer, trace_id, Idempotency-Key.
 *
 * All outgoing requests MUST include `x-trace-id` (every response carries
 * one back). Mutations MUST additionally include `Idempotency-Key`.
 */

import { generateTraceId, generateIdempotencyKey } from '@lcc/api-types/brand';

let sessionTraceId: string | null = null;

/** Get or set the current session trace ID (one per page lifecycle). */
export function getSessionTraceId(): string {
  if (typeof window !== 'undefined' && window.crypto?.randomUUID) {
    if (!sessionTraceId) sessionTraceId = window.crypto.randomUUID();
    return sessionTraceId;
  }
  if (!sessionTraceId) sessionTraceId = generateTraceId();
  return sessionTraceId;
}

export function resetSessionTraceId(): void {
  sessionTraceId = null;
}

export interface AuthHeadersOptions {
  /** Generate a fresh trace ID rather than reusing the session one. */
  freshTraceId?: boolean;
  /** For mutations — pass a key or auto-generate. */
  idempotencyKey?: string;
}

export function buildAuthHeaders(
  token: string | null,
  opts: AuthHeadersOptions = {},
): Record<string, string> {
  const headers: Record<string, string> = {
    'x-trace-id': opts.freshTraceId ? generateTraceId() : getSessionTraceId(),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  if (opts.idempotencyKey) {
    headers['Idempotency-Key'] = opts.idempotencyKey;
  }
  return headers;
}

export function freshIdempotencyKey(): string {
  return generateIdempotencyKey();
}
