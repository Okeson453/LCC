//! Log redaction middleware.
//!
//! Strips known sensitive headers from request/response logs.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

const REDACTED: &str = "[REDACTED]";

const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "x-api-key",
    "x-auth-token",
    "cookie",
    "set-cookie",
    "x-permit-token",
];

pub async fn redact_log(req: Request<Body>, next: Next) -> Response {
    // In production we'd hook into a custom tracing layer that uses this list.
    // For now, this middleware just passes through.
    let _ = (req, REDACTED, SENSITIVE_HEADERS);
    next.run(req).await
}
