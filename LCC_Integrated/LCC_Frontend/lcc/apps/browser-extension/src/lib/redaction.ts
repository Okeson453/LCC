/**
 * Redact identifiers like access tokens, JWTs, and PII prior to logging.
 * The extension only ever logs redacted strings.
 */

const TOKEN_RX = /Bearer\s+[A-Za-z0-9\-._~+/]+=*/g;
const JWT_RX = /eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9_-]+/g;
const EMAIL_RX = /[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g;

export function redactAccessTokens(s: string): string {
  return s.replace(TOKEN_RX, 'Bearer [REDACTED]').replace(JWT_RX, '[JWT_REDACTED]');
}

export function redactPII(s: string): string {
  return redactAccessTokens(s).replace(EMAIL_RX, '[EMAIL_REDACTED]');
}
