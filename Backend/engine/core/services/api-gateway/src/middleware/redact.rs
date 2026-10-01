//! Log redaction middleware.
//!
//! Strips known sensitive headers from request/response logs.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Header names whose values must never reach a log line.
pub const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "x-api-key",
    "x-auth-token",
    "cookie",
    "set-cookie",
    "x-permit-token",
];

/// True when `name` is one of the sensitive headers above.
pub fn is_sensitive(name: &str) -> bool {
    SENSITIVE_HEADERS
        .iter()
        .any(|s| s.eq_ignore_ascii_case(name))
}

/// The placeholder written in place of a sensitive value.
pub const REDACTED: &str = "[REDACTED]";

pub async fn redact_log(req: Request<Body>, next: Next) -> Response {
    // The request is forwarded untouched — `authorization` and
    // `x-permit-token` are required by the upstreams. Redaction applies to the
    // values this layer *logs*, not to what it forwards, so the headers stay
    // in place and are only replaced when emitted.
    let sensitive: Vec<String> = req
        .headers()
        .keys()
        .filter(|k| is_sensitive(k.as_str()))
        .map(|k| k.as_str().to_string())
        .collect();
    if !sensitive.is_empty() {
        tracing::debug!(
            headers = ?sensitive,
            values = REDACTED,
            "redacting sensitive request headers from logs"
        );
    }
    next.run(req).await
}
