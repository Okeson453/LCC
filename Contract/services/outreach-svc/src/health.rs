use axum::{routing::get, Json, Router};
use serde_json::json;
pub fn router() -> Router {
    Router::new().route("/health", get(|| async { Json(json!({"status":"ok"})) }))
}
