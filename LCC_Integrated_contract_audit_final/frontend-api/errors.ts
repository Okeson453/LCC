/**
 * API error taxonomy.
 */

export type ApiErrorKind =
  | 'network'
  | 'timeout'
  | 'unauthorized'
  | 'forbidden'
  | 'not_found'
  | 'validation'
  | 'rate_limited'
  | 'restricted'
  | 'governance_denied'
  | 'server_error'
  | 'version_mismatch'
  | 'unknown';

export class ApiError extends Error {
  readonly kind: ApiErrorKind;
  readonly status: number;
  readonly traceId: string | null;
  readonly details?: unknown;

  constructor(input: {
    kind: ApiErrorKind;
    message: string;
    status: number;
    traceId: string | null;
    details?: unknown;
  }) {
    super(input.message);
    this.name = 'ApiError';
    this.kind = input.kind;
    this.status = input.status;
    this.traceId = input.traceId;
    this.details = input.details;
  }
}

export function classifyError(err: unknown): ApiError {
  if (err instanceof ApiError) return err;
  if (err instanceof DOMException && err.name === 'AbortError') {
    return new ApiError({ kind: 'timeout', message: 'Request timed out', status: 0, traceId: null });
  }
  if (err instanceof TypeError) {
    return new ApiError({ kind: 'network', message: 'Network error', status: 0, traceId: null });
  }
  return new ApiError({
    kind: 'unknown',
    message: err instanceof Error ? err.message : 'Unknown error',
    status: 0,
    traceId: null,
  });
}

export function isRetryable(err: ApiError): boolean {
  return err.kind === 'network' || err.kind === 'timeout' || err.kind === 'server_error' || err.kind === 'rate_limited';
}
