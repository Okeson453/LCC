//! api-gateway health endpoints.
//!
//! F-AUDIT-11: `readyz` previously returned `{"status":"ready"}`
//! unconditionally, without contacting a single upstream. Kubernetes uses this
//! endpoint as the readiness probe, so a gateway pointed at a completely dead
//! set of upstreams was still routed traffic. `readyz` now performs a real
//! reachability check against each bound upstream and fails (503) if any
//! required one is unreachable.
//!
//! `healthz` remains a pure liveness check — it must not depend on downstream
//! state, or a downstream outage would cause Kubernetes to restart-loop every
//! gateway pod and turn a partial outage into a total one.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;

use crate::proxy::pool::SharedPool;
use crate::state::AppState;

/// Per-upstream probe budget. Kept short so a slow downstream cannot stall the
/// readiness probe past its own `timeoutSeconds` and cause a restart loop.
const PROBE_TIMEOUT: Duration = Duration::from_millis(750);

/// Returns the health routes in the gateway's own `AppState`, so they can be
/// merged with the protected/proxy routers before `.with_state(..)` fixes the
/// state type.
///
/// Includes `GET /metrics`: the canonical contract declares it as a public
/// scrape endpoint (`security: []`) and `docs/endpoint_contract_matrix.md`
/// lists it as served by the gateway, but it was never registered here, so a
/// Prometheus scrape of the gateway returned 404.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
}

/// Prometheus exposition of the gateway's own metrics registry.
///
/// Deliberately unauthenticated: Prometheus scrapes carry no Bearer token, and
/// the contract marks this operation `security: []`. It exposes only
/// process-level counters and histograms — never request bodies, member ids or
/// credentials.
async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    match state.metrics().render() {
        Ok(body) => (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            body,
        )
            .into_response(),
        Err(e) => {
            tracing::warn!("failed to render metrics exposition: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                "metrics unavailable\n".to_string(),
            )
                .into_response()
        }
    }
}

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "api-gateway" }))
}

async fn readyz(State(state): State<AppState>) -> (axum::http::StatusCode, Json<Value>) {
    let mut checks: Vec<Value> = Vec::new();
    let mut all_ok = true;

    for prefix in state.upstreams().bound_prefixes() {
        match state.upstreams().pool_for(prefix) {
            Some(pool) => {
                let ok = probe(pool).await;
                all_ok &= ok;
                checks.push(json!({ "domain": prefix, "ok": ok }));
            }
            None => {
                all_ok = false;
                checks.push(json!({ "domain": prefix, "ok": false, "error": "no upstream bound" }));
            }
        }
    }

    let status = if all_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if all_ok { "ready" } else { "not_ready" },
            "service": "api-gateway",
            "upstreams": checks,
        })),
    )
}

/// Probe a single upstream's health endpoint. Returns false on any error or
/// timeout — readiness is fail-closed so a bad dependency surfaces.
async fn probe(pool: &SharedPool) -> bool {
    let url = format!("{}/healthz", pool.config.base_url);
    match timeout(PROBE_TIMEOUT, pool.http.get(&url).send()).await {
        Ok(Ok(resp)) => resp.status().is_success(),
        Ok(Err(_)) => false,
        Err(_) => false, // probe timed out
    }
}
