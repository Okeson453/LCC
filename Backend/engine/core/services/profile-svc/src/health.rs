//! Health, readiness and metrics endpoints.
//!
//! F-AUDIT-56: this router previously served only `/health`, while every
//! Kubernetes manifest in `infra/k8s/base` probes `/healthz` (liveness) and
//! `/readyz` (readiness). Those pods would have failed their probes, been
//! killed and crash-looped forever. The Backend Design Concept states the
//! requirement directly: "Every service has `/healthz` + `/readyz` +
//! `/metrics` endpoints".
//!
//! `/healthz` is a pure liveness check and deliberately touches no dependency:
//! a transient database blip must not cause Kubernetes to restart an otherwise
//! healthy pod. `/readyz` is what actually gates traffic.
//!
//! The legacy `/health` and `/ready` paths are kept as aliases so any existing
//! caller (docker-compose healthchecks, the gateway's upstream probe) keeps
//! working while the canonical paths become the served contract.

use axum::{http::StatusCode, routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // Backwards-compatible aliases (pre-F-AUDIT-56 paths).
        .route("/health", get(healthz))
        .route("/ready", get(readyz))
}

/// Liveness: the process is running. Must not depend on Postgres or Redis.
async fn healthz() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

/// Readiness: this service has no mandatory startup dependency beyond the
/// listener itself, so it is ready once the process is serving.
async fn readyz() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "ready"})))
}
