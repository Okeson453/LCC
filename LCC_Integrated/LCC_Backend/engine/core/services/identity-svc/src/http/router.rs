//! identity-svc canonical HTTP router.

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{http::handlers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::health::router())
        // Canonical namespace per contract_audit/openapi/lcc-api-canonical.yaml
        .route(
            "/api/v1/auth/linkedin/start",
            get(handlers::auth_linkedin_start),
        )
        .route(
            "/api/v1/auth/linkedin/callback",
            get(handlers::auth_linkedin_callback),
        )
        .route("/api/v1/auth/refresh", post(handlers::auth_refresh))
        .route("/api/v1/auth/logout", post(handlers::auth_logout))
        .route("/api/v1/members/me", get(handlers::members_me))
        .route(
            "/api/v1/members/me/settings",
            get(handlers::members_me_settings_get).patch(handlers::members_me_settings_patch),
        )
        .route(
            "/api/v1/members/:member_id",
            get(handlers::members_get),
        )
        .with_state(state)
}
