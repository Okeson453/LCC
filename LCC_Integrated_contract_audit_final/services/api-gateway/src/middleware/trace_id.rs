//! Trace-id propagation middleware.

use axum::{
    body::Body,
    http::{HeaderValue, Request, Response},
    middleware::Next,
};

pub const HEADER: &str = "x-trace-id";

/// Adds a trace_id to the request extensions if one isn't already set.
pub async fn propagate_trace_id(mut req: Request<Body>, next: Next) -> Response {
    let trace_id = req
        .headers()
        .get(HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| {
            // Generate a 16-byte hex id.
            use rand::Rng;
            let bytes: [u8; 16] = rand::thread_rng().gen();
            hex::encode(bytes)
        });

    req.extensions_mut().insert(trace_id.clone());

    let mut response: Response = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert(HEADER, value);
    }
    response
}
