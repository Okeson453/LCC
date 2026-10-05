/**
 * Redact identifiers like access tokens, JWTs, and PII prior to logging.
 * The extension only ever logs redacted strings.
 */

const TOKEN_RX = /Bearer\s+[A-Za-z0-9\-._~+/]+=*/g;
// A JWT is `base64url(header).base64url(payload).base64url(signature)`. The
// header is a JSON object, so it always base64-encodes with the `eyJ` prefix;
// the payload and signature are opaque and must NOT be assumed to share that
// prefix. The previous pattern required the *payload* segment to start with
// `eyJ` too, so any token whose payload was not a `{"`-prefixed object was
// passed through unredacted — a fail-open in a redaction control. Only the
// header is constrained; the other two segments are any base64url run.
const JWT_RX = /eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9_-]+/g;
const EMAIL_RX = /[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g;

export function redactAccessTokens(s: string): string {
  return s.replace(TOKEN_RX, 'Bearer [REDACTED]').replace(JWT_RX, '[JWT_REDACTED]');
}

export function redactPII(s: string): string {
  return redactAccessTokens(s).replace(EMAIL_RX, '[EMAIL_REDACTED]');
}
