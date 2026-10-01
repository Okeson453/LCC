//! identity-svc health endpoints.

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

async fn healthz() -> Json<Value> { Json(json!({ "status": "ok" })) }
async fn readyz() -> Json<Value> { Json(json!({ "status": "ready" })) }
