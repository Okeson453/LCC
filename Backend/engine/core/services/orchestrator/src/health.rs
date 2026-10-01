//! Health endpoints.

use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

async fn ready() -> Json<Health> {
    Json(Health { status: "ready" })
}
