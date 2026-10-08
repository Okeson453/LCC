//! Orchestrator canonical HTTP router.

use axum::{
    routing::{get, post},
    Router,
};

use crate::{http_handlers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    // Sub-routers share `AppState`; `.with_state(state)` erases it at the end.
    Router::<AppState>::new()
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
        // F-AUDIT-56: the health router is stateless, so it is merged *after*
        // `.with_state()`. Merging it into a `Router<AppState>` would pin the
        // state type to `()` and stop every stateful route from type-checking.
        // It serves `/healthz` and `/readyz`, which is what every manifest in
        // infra/k8s/base probes.
        .merge(crate::health::router())
}
