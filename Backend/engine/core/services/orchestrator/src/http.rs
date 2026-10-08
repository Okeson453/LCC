//! Orchestrator canonical HTTP router.

use axum::{
    routing::{get, post},
    Router,
};
use lcc_auth::{Permission, SharedVerifier};

use crate::{http_handlers, state::AppState};

/// The briefing routes are member-scoped, so they are gated on
/// [`Permission::ViewOwnData`] rather than left open.
///
/// Before this pass the router applied no authentication at all: the handlers
/// took `member_id` straight from the path and never read the `Authorization`
/// header, so any caller who could reach the service could read any member's
/// daily briefing by editing one path segment. The gateway verifies a token
/// before proxying, but it does not compare the path's `:member_id` against the
/// token subject, and this service trusted the path.
///
/// The gate is a `route_layer` applied where the routes are registered, not a
/// per-handler check, so a route added later is protected by default rather
/// than by remembering. 401 when there is no usable credential; 403 when the
/// caller's role lacks the permission.
pub fn build_router(state: AppState, verifier: SharedVerifier) -> Router {
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
        // Every route registered so far requires an authenticated caller that
        // holds ViewOwnData. The health routes merged below are deliberately
        // outside this layer: the kubelet probes them with no bearer token.
        .route_layer(axum::middleware::from_fn(move |request, next| {
            let v = verifier.clone();
            async move {
                lcc_auth::guard::require_permission(v, Permission::ViewOwnData, request, next).await
            }
        }))
        .with_state(state)
        // F-AUDIT-56: the health router is stateless, so it is merged *after*
        // `.with_state()`. Merging it into a `Router<AppState>` would pin the
        // state type to `()` and stop every stateful route from type-checking.
        // It serves `/healthz` and `/readyz`, which is what every manifest in
        // infra/k8s/base probes.
        .merge(crate::health::router())
}
