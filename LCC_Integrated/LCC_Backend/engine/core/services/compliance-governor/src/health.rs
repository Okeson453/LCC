//! Health endpoints for the Compliance Governor.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use lcc_observability::metrics::Metrics;
use serde::Serialize;
use std::sync::Arc;

use crate::state::GovernorDeps;

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[derive(Serialize)]
pub struct ReadyzReport {
    pub status: &'static str,
    pub db_ok: bool,
    pub redis_ok: bool,
    pub active_compliance_config_version: String,
    pub uptime_seconds: u64,
}

pub async fn readyz(
    State(deps): State<Arc<GovernorDeps>>,
    started_at: axum::extract::State<std::time::Instant>,
) -> impl IntoResponse {
    // Check DB.
    let db_ok = sqlx::query("SELECT 1").execute(&deps.db).await.is_ok();
    // Check Redis.
    let redis_ok = if let Ok(mut conn) = deps.redis.get().await {
        let _: redis::RedisResult<i64> = redis::cmd("PING").query_async(&mut *conn).await;
        true
    } else {
        false
    };

    let report = ReadyzReport {
        status: if db_ok && redis_ok { "ready" } else { "degraded" },
        db_ok,
        redis_ok,
        active_compliance_config_version: deps.config.version.clone(),
        uptime_seconds: started_at.0.elapsed().as_secs(),
    };

    if db_ok && redis_ok {
        (StatusCode::OK, Json(report)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(report)).into_response()
    }
}

pub async fn metrics_handler(State(metrics): State<Arc<Metrics>>) -> impl IntoResponse {
    match metrics.render() {
        Ok(text) => (StatusCode::OK, [("content-type", "text/plain; version=0.0.4")], text)
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "metrics render failed").into_response(),
    }
}

pub static STARTED_AT: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub fn started_at() -> std::time::Instant {
    *STARTED_AT.get_or_init(std::time::Instant::now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn started_at_is_monotonic() {
        let s1 = started_at();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let s2 = started_at();
        assert!(s2 > s1);
    }
}
