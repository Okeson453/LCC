/**
 * Mock logger — drops PII / tokens / LinkedIn credentials before logging.
 *
 * Used in tests AND in the production app's dev-mode logger.
 */

const SENSITIVE_KEYS = [
  'password',
  'token',
  'access_token',
  'refresh_token',
  'idempotency_key',
  'authorization',
  'linkedin',
  'session_state',
  'code',
  'cookie',
];

const REDACTED = '[REDACTED]';

function redactValue(value: unknown): unknown {
  if (value == null) return value;
  if (typeof value === 'string') {
    if (/Bearer\s+[A-Za-z0-9_-]+\.?[A-Za-z0-9_-]*\.?[A-Za-z0-9_-]*/.test(value)) {
      return REDACTED;
    }
    if (/linkedin\.com/.test(value)) return REDACTED;
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(redactValue);
  }
  if (typeof value === 'object') {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      const lower = k.toLowerCase();
      if (SENSITIVE_KEYS.some((s) => lower.includes(s))) {
        out[k] = REDACTED;
      } else {
        out[k] = redactValue(v);
      }
    }
    return out;
  }
  return value;
}

export function redact(input: unknown): unknown {
  return redactValue(input);
}

export const logger = {
  info(message: string, meta?: Record<string, unknown>) {
    if (typeof console !== 'undefined') {
      // eslint-disable-next-line no-console
      console.info(message, redact(meta));
    }
  },
  warn(message: string, meta?: Record<string, unknown>) {
    if (typeof console !== 'undefined') {
      // eslint-disable-next-line no-console
      console.warn(message, redact(meta));
    }
  },
  error(message: string, meta?: Record<string, unknown>) {
    if (typeof console !== 'undefined') {
      // eslint-disable-next-line no-console
      console.error(message, redact(meta));
    }
  },
};
