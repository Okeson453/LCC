//! Orchestrator canonical HTTP router.

use axum::{
    routing::{get, post},
    Router,
};

use crate::{http_handlers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::health::router())
        // Canonical namespace per lcc-api-canonical.yaml
        .route(
            "/api/v1/members/:member_id/briefing/today",
            get(http_handlers::briefing_today),
        )
        .route(
            "/api/v1/members/:member_id/briefing/refresh",
            post(http_handlers::briefing_refresh),
        )
        .with_state(state)
}
