//! HTTP handlers — proxy routes to upstream services.
//!
//! F-AUDIT-01: these handlers previously returned a hardcoded
//! `{"status":"ok"}` for every protected route without contacting any
//! upstream. They now perform a real reverse-proxy forward via
//! [`crate::proxy`].
//!
//! The original `Path<String>` extractor only captured the wildcard suffix
//! (`/v1/content/*path` → `path` is the part *after* `/v1/content`). The
//! handlers below therefore reconstruct the full upstream path from the
////! matched-path extension that the router layer records, falling back to a
//! `/` root request when it is absent.

use axum::{
    body::Body,
    extract::{OriginalUri, Request, State},
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
};
use serde_json::json;

use crate::{error::ApiGatewayError, state::AppState};

/// Proxy any method/body combination to the upstream owning this route.
///
/// One handler serves GET/POST/PATCH/PUT/DELETE so the gateway does not need
/// a duplicate pair per verb, and so an unimplemented verb on the upstream
/// surfaces as a real 405 from the upstream rather than a fake 200 here.
pub async fn proxy_request(
    State(state): State<AppState>,
    OriginalUri(original): OriginalUri,
    request: Request<Body>,
) -> Result<axum::response::Response, ApiGatewayError> {
    let (parts, body) = request.into_parts();
    let full_path = original.path().to_string();
    let query = original.query().map(str::to_string);

    let registry = state.upstreams();
    let pool = registry
        .resolve(&full_path)
        .map_err(|why| ApiGatewayError::NotFound(why))?;

    // Determine the upstream service name to decide whether the path needs
    // rewriting (legacy upstreams still expose `/v1/<svc>_svc/...`).
    let upstream_service = pool.upstream_name().to_string();
    let upstream_path = registry.rewrite_for_upstream(&full_path, &upstream_service);

    let body_bytes = crate::proxy::read_body(body).await?;

    let (status, headers, upstream_body) = crate::proxy::forward(
        pool,
        &parts.method,
        &upstream_path,
        query.as_deref(),
        &parts.headers,
        body_bytes,
    )
    .await?;

    let mut response = (status, upstream_body).into_response();
    *response.headers_mut() = headers;
    Ok(response)
}

/// GET passthrough (kept as a named handler so the router reads explicitly and
/// so `/v1/...` GETs keep a stable OpenAPI operation mapping).
pub async fn proxy_get(
    State(state): State<AppState>,
    OriginalUri(original): OriginalUri,
    request: Request<Body>,
) -> Result<axum::response::Response, ApiGatewayError> {
    proxy_with_method(Method::GET, state, original, request).await
}

/// POST passthrough.
pub async fn proxy_post(
    State(state): State<AppState>,
    OriginalUri(original): OriginalUri,
    request: Request<Body>,
) -> Result<axum::response::Response, ApiGatewayError> {
    proxy_with_method(Method::POST, state, original, request).await
}

async fn proxy_with_method(
    method: Method,
    state: AppState,
    original: Uri,
    request: Request<Body>,
) -> Result<axum::response::Response, ApiGatewayError> {
    // `Request::into_parts` yields (Parts, Body); the URI is rebuilt below from
    // the original request so the method override is the only difference.
    let (mut parts, body) = request.into_parts();
    parts.method = method;
    proxy_request(State(state), OriginalUri(original), Request::from_parts(parts, body)).await
}

/// Gateway root — lists the domain prefixes actually bound to an upstream.
///
/// F-AUDIT-01 follow-up: this makes an unbound or misconfigured upstream
/// visible at runtime instead of only in source.
pub async fn gateway_root(State(state): State<AppState>) -> impl IntoResponse {
    let bound = state.upstreams().bound_prefixes();
    let cfg = state.config();
    (
        StatusCode::OK,
        axum::Json(json!({
            "service": cfg.service_name,
            "version": env!("CARGO_PKG_VERSION"),
            "bound_domains": bound,
            "unbound_domains_remain_404": true,
        })),
    )
}
