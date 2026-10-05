//! Health endpoints for the Integration Gateway.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::state::IntegrationGatewayState;

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[derive(Serialize)]
pub struct ReadyzReport {
    pub status: &'static str,
    pub db_ok: bool,
    pub redis_ok: bool,
}

pub async fn readyz(State(state): State<Arc<IntegrationGatewayState>>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let redis_ok = if let Ok(mut conn) = state.redis.get().await {
        let _: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut *conn).await;
        true
    } else {
        false
    };
    let report = ReadyzReport {
        status: if db_ok && redis_ok {
            "ready"
        } else {
            "degraded"
        },
        db_ok,
        redis_ok,
    };
    let status = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(report))
}
