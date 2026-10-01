use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new().route("/health", get(|| async { Json(json!({"status":"ok"})) }))
}

#[allow(dead_code)]
fn _t() -> Value {
    json!({})
}
