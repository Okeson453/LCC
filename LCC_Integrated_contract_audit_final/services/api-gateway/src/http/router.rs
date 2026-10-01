//! Router — assembles all routes for the api-gateway.
//!
//! Canonical public surface per lcc-api-canonical.yaml:
//!   - `/api/v1/auth/*`        OAuth2/PKCE handshake, refresh, logout
//!   - `/api/v1/members/*`     Member account self-service
//!   - `/api/v1/profile/*`     Profile snapshots, audit, edit-drafts (per-member)
//!   - `/api/v1/content/*`     ContentItem lifecycle (per-member)
//!   - `/api/v1/engagement/*`  Engagement queue + reply drafts (per-member)
//!   - `/api/v1/contacts/*`    Network/CRM contacts + interactions
//!   - `/api/v1/opportunities/*` Opportunity discovery + scoring
//!   - `/api/v1/sequences/*`   Outreach sequence lifecycle
//!   - `/api/v1/kb/*`          Professional Knowledge Base
//!   - `/api/v1/analytics/*`   Dashboards + digests
//!   - `/api/v1/briefing/*`    Briefing assembly
//!   - `/api/v1/approvals/*`   Approval queue + decisions
//!   - `/api/v1/audit/*`       Audit log reads
//!   - `/api/v1/admin/*`       Compliance config, restrictions
//!   - `/api/v1/ws/*`          WebSocket channels (proxied to realtime-svc)
//!   - `/healthz`, `/readyz`, `/metrics`
//!
//! Auth gating is layer-wide so every protected path requires a valid
//! Bearer JWT. The legacy `/v1/<svc>_svc/...` paths are no longer served.

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers,
    http::handlers as proxy_h,
    middleware::{
        auth::require_auth,
        rate_limit::rate_limit,
        trace_id::propagate_trace_id,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/api/v1/auth/linkedin/start", get(handlers::auth_start::auth_start))
        .route(
            "/api/v1/auth/linkedin/callback",
            get(handlers::auth_callback::auth_callback),
        )
        .route("/api/v1/auth/refresh", post(handlers::auth_refresh::auth_refresh))
        .route("/api/v1/auth/logout", post(handlers::auth_logout::auth_logout));

    // Protected domain routes. The proxy forwarder passes the canonical path
    // verbatim to the upstream — every upstream implements the canonical
    // namespace `/api/v1/<domain>/...` natively, so no rewrite is needed.
    let protected = Router::new()
        .route(
            "/api/v1/members/me",
            get(proxy_h::proxy_request).patch(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/members/me/settings",
            get(proxy_h::proxy_request).patch(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/members/:member_id",
            get(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/members/:member_id/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/profile/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/content/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/engagement/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/contacts/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/opportunities/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/sequences/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/kb/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/analytics/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/briefing/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/approvals/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/audit/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/admin/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        // Realtime — proxied to realtime-svc.
        .route(
            "/api/v1/ws/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        .route(
            "/api/v1/sse/*path",
            axum::routing::any(proxy_h::proxy_request),
        )
        // Legacy governance eval path kept for internal callers during the
        // migration window; will be removed once the gateway is the sole
        // caller. The canonical frontend path is the per-resource
        // `/approvals/{id}/decide` flow.
        .route(
            "/api/v1/admin/governor/evaluate",
            post(proxy_h::proxy_request),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ))
        // F-AUDIT-05: rate limiting existed as a module but was never
        // layered onto the router, and constructed a fresh counter per
        // request, so it could not have limited anything. It is now layered
        // and holds shared per-process state.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit,
        ));

    Router::new()
        .merge(crate::health::router())
        // Public auth routes (start/callback are unauthenticated;
        // refresh/logout handle their own auth).
        .merge(auth_routes)
        .merge(protected)
        // Trace-id propagation applies to every route so the id is present
        // on upstream forwards and on the client-visible response.
        .layer(middleware::from_fn(propagate_trace_id))
        .with_state(state)
}
