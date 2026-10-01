//! Health endpoints.

use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "lcc-realtime-svc" }))
}

async fn readyz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ready", "service": "lcc-realtime-svc" }))
}
