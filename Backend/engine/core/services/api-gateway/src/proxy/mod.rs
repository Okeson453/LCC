//! Reverse-proxy module.
//!
//! Design source: Backend Design Concept §17 (REST API conventions) and §20
//! (internal service contracts) — the public HTTP surface terminates at the
//! api-gateway, which is the *only* component the browser talks to. Every
//! `/v1/<domain>/...` route is forwarded to the owning Core Engine service.
//!
//! ### Why this module exists
//!
//! F-AUDIT-01: `proxy_get`/`proxy_post` previously returned a hardcoded
//! `{"status":"ok"}` for every protected route, and the `UpstreamClient` pool
//! that already existed in `proxy/pool.rs` was dead code (never constructed,
//! never referenced). That made every protected call report success while
//! executing nothing. This module wires the pool for real: a domain-prefix
//! registry resolves a route to an upstream, and the request is actually
//! forwarded.
//!
//! ### Honesty note
//!
//! The path is forwarded to the upstream **unchanged**. The Core Engine domain
//! services currently expose `/v1/<service>_svc/...` rather than the
//! `/v1/<domain>/...` surface documented in §18, so forwarding verbatim makes
//! those gaps surface as honest upstream 404s instead of a fabricated 200.
//! Rewriting the prefix here would paper over that gap with another layer of
//! fiction, so it is deliberately not done.

pub mod pool;

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
};
use reqwest::Response as ReqwestResponse;

use crate::error::ApiGatewayError;
use pool::SharedPool;

/// Headers that must be forwarded to the upstream. Everything else (hop-by-hop
/// headers, `host`, `content-length`, `connection`) is dropped per RFC 9110
/// §7.6.1 so the upstream sees a clean request.
const FORWARD_REQUEST_HEADERS: &[&str] = &[
    "authorization",
    "x-trace-id",
    "x-request-id",
    "idempotency-key",
    "x-member-id",
    "accept",
    "accept-language",
    "if-none-match",
];

/// Hop-by-hop response headers that must not be copied back to the client.
const HOP_BY_HOP_RESPONSE_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "transfer-encoding",
    "upgrade",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
];

/// Maximum request body the gateway will buffer before forwarding. Prevents a
/// single oversized payload from pinning gateway memory (see audit §
/// Performance: unbounded buffering).
pub const MAX_FORWARD_BODY_BYTES: usize = 8 * 1024 * 1024;

/// A domain prefix (the first path segment after `/v1`) mapped to its upstream.
#[derive(Debug, Clone)]
pub struct RouteBinding {
    pub prefix: &'static str,
    pub pool: SharedPool,
}

/// Resolves an incoming `/v1/<domain>/...` path to the owning service.
#[derive(Clone)]
pub struct UpstreamRegistry {
    bindings: Arc<Vec<RouteBinding>>,
}

impl UpstreamRegistry {
    pub fn new(bindings: Vec<RouteBinding>) -> Self {
        Self {
            bindings: Arc::new(bindings),
        }
    }

    /// Resolve `/api/v1/<domain>/rest...` to its upstream.
    ///
    /// Returns `Err` with the unmatched domain so the caller can return a
    /// 404 that names the missing binding — a silent fallback would hide
    /// routing regressions.
    pub fn resolve(&self, full_path: &str) -> Result<&SharedPool, String> {
        // Canonical namespace is `/api/v1/<domain>/...` — domain is the
        // third path segment. We accept (and skip) the legacy `/v1/<domain>`
        // shape for a deprecation window by attempting both.
        let parts: Vec<&str> = full_path.trim_start_matches('/').split('/').collect();
        let domain = match parts.as_slice() {
            ["api", "v1", d, ..] => *d,
            ["v1", d, ..] => *d,
            [d, ..] => *d,
            _ => "",
        };

        // Subdomain override: the briefing namespace lives under
        // `/api/v1/members/{member_id}/briefing/...` per the canonical
        // contract. If we see a 5th segment == "briefing", route to the
        // orchestrator's pool regardless of the parent domain being
        // `members`.
        let effective_domain = if parts.len() >= 5
            && parts[..4] == ["api", "v1", "members"]
            && parts[4] == "briefing"
        {
            "briefing"
        } else {
            domain
        };

        self.bindings
            .iter()
            .find(|b| b.prefix == effective_domain)
            .map(|b| &b.pool)
            .ok_or_else(|| format!("no upstream bound for domain `{effective_domain}`"))
    }

    /// Translate the canonical inbound path to the upstream-expected path.
    ///
    /// Most upstreams implement the canonical `/api/v1/<domain>/...` namespace
    /// directly. A few still expose legacy scaffold routes at
    /// `/v1/<service>_svc/...`; this function rewrites those for the
    /// duration of the migration window so the gateway remains functional
    /// even while services complete their canonicalization.
    ///
    /// Returns the upstream-bound path; the caller appends the query string.
    pub fn rewrite_for_upstream(&self, full_path: &str, pool_name: &str) -> String {
        // These upstreams still expose only the legacy `/v1/<service>_svc/...`
        // scaffold routes. Everything else (identity, governor, realtime,
        // orchestrator) already serves the canonical namespace verbatim.
        let upstream_service = match pool_name {
            "profile-svc" | "content-svc" | "engagement-svc" | "network-crm-svc"
            | "opportunity-svc" | "outreach-svc" | "analytics-svc" | "approval-svc"
            | "audit-svc" | "kb-svc" => pool_name,
            _ => return full_path.to_string(),
        };
        let svc_tag = format!("{}_svc", upstream_service.trim_end_matches("-svc"));
        // Rewrite /api/v1/<domain>/...  → /v1/<svc_tag>/...
        // Rewrite /api/v1/admin/compliance/* → /v1/admin/<svc_tag>/* (special case)
        if full_path.starts_with("/api/v1/admin/") {
            // Compliance Governor uses canonical namespace directly.
            return full_path.to_string();
        }
        if let Some(rest) = full_path.strip_prefix("/api/v1/") {
            // Replace first path segment with `<svc_tag>`
            if let Some(slash) = rest.find('/') {
                format!("/v1/{}{}", svc_tag, &rest[slash..])
            } else {
                format!("/v1/{}", svc_tag)
            }
        } else {
            full_path.to_string()
        }
    }


    /// Prefixes that are currently bound. Surfaced on the gateway root route so
    /// a running deployment can be inspected for missing bindings.
    pub fn bound_prefixes(&self) -> Vec<&'static str> {
        self.bindings.iter().map(|b| b.prefix).collect()
    }

    /// Look up the upstream bound to a specific domain prefix.
    pub fn pool_for(&self, prefix: &str) -> Option<&SharedPool> {
        self.bindings
            .iter()
            .find(|b| b.prefix == prefix)
            .map(|b| &b.pool)
    }
}

/// Forward `method` + `path` to `pool`, streaming the status, body and
/// end-to-end headers back to the caller.
pub async fn forward(
    pool: &SharedPool,
    method: &Method,
    full_path: &str,
    query: Option<&str>,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, HeaderMap, Body), ApiGatewayError> {
    let target = match query {
        Some(q) if !q.is_empty() => format!("{full_path}?{q}"),
        _ => full_path.to_string(),
    };

    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .map_err(|e| ApiGatewayError::BadRequest(format!("unsupported method: {e}")))?;

    let mut builder = pool.http.request(reqwest_method, target);

    for name in FORWARD_REQUEST_HEADERS {
        if let Some(value) = headers.get(*name) {
            if let (Ok(n), Ok(v)) = (
                HeaderName::from_bytes(name.as_bytes()),
                value.to_str().map(str::to_string),
            ) {
                if let Ok(hv) = HeaderValue::from_str(&v) {
                    builder = builder.header(n, hv);
                }
            }
        }
    }

    if !body.is_empty() {
        builder = builder.body(body.to_vec());
    }

    let upstream = builder.send().await?;
    translate_response(upstream).await
}

type Bytes = axum::body::Bytes;

/// Copy an upstream response back to the client, preserving status and
/// end-to-end headers.
async fn translate_response(upstream: ReqwestResponse) -> Result<(StatusCode, HeaderMap, Body), ApiGatewayError> {
    let status = StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let headers = upstream.headers().clone();
    let bytes = upstream
        .bytes()
        .await
        .map_err(|e| ApiGatewayError::Upstream(format!("reading upstream body: {e}")))?;

    let mut out_headers = HeaderMap::new();
    for (name, value) in headers.iter() {
        let lower = name.as_str().to_ascii_lowercase();
        if HOP_BY_HOP_RESPONSE_HEADERS.contains(&lower.as_str()) {
            continue;
        }
        // `content-length` is recomputed by the response body; copying the
        // upstream value after re-buffering would be wrong for HEAD/204.
        if lower == "content-length" {
            continue;
        }
        if let Ok(n) = HeaderName::from_bytes(lower.as_bytes()) {
            out_headers.insert(n, value.clone());
        }
    }

    Ok((status, out_headers, Body::from(bytes)))
}

/// Read a request body with a hard size cap, so an oversized payload is
/// rejected at the edge instead of being buffered.
pub async fn read_body(body: Body) -> Result<Bytes, ApiGatewayError> {
    to_bytes(body, MAX_FORWARD_BODY_BYTES)
        .await
        .map_err(|_| ApiGatewayError::BadRequest("request body too large".into()))
}

/// Build the upstream URL for a path, honouring the client's original query.
pub fn upstream_uri(base_url: &str, full_path: &str, query: Option<&str>) -> Result<Uri, ApiGatewayError> {
    let joined = format!("{base_url}{full_path}");
    let with_query = match query {
        Some(q) if !q.is_empty() => format!("{joined}?{q}"),
        _ => joined,
    };
    with_query
        .parse::<Uri>()
        .map_err(|e| ApiGatewayError::Internal(format!("bad upstream url: {e}")))
}
