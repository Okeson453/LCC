//! Router — assembles all routes for the api-gateway.
//!
//! The routing surface itself is declared as data in [`crate::routes`]; this
//! module only turns that table into an `axum::Router`. Keeping the table
//! separate from the builder is what makes the surface verifiable against
//! `Contract/openapi/lcc-api-canonical.yaml`
//! (`tests/gateway_contract_conformance.rs`).
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
    middleware::{auth::require_auth, rate_limit::rate_limit, trace_id::propagate_trace_id},
    routes::{self, Method},
    state::AppState,
};

/// Build the infra sub-router: served in-process, never auth-gated.
fn build_infra_router() -> Router<AppState> {
    let mut infra = Router::<AppState>::new();
    for spec in routes::infra_routes() {
        infra = infra.route(
            spec.pattern,
            match spec.pattern {
                "/healthz" => get(crate::health::healthz),
                "/readyz" => get(crate::health::readyz),
                "/metrics" => get(crate::health::metrics),
                // Unreachable: `routes::ROUTES` is the single source of truth
                // and the conformance test asserts the set of infra patterns.
                other => {
                    debug_assert!(false, "no handler for infra route {other}");
                    axum::routing::any(crate::health::unregistered_infra)
                }
            },
        );
    }
    infra
}

pub fn build_router(state: AppState) -> Router {
    // Infra: served in-process, no auth.
    let infra = build_infra_router();

    // Public OAuth entry points (no Bearer token exists yet at handshake time).
    let mut auth_routes = Router::<AppState>::new();
    for spec in routes::public_auth_routes() {
        auth_routes = auth_routes.route(
            spec.pattern,
            match spec.pattern {
                "/api/v1/auth/linkedin/start" => get(handlers::auth_start::auth_start),
                "/api/v1/auth/linkedin/callback" => get(handlers::auth_callback::auth_callback),
                "/api/v1/auth/refresh" => post(handlers::auth_refresh::auth_refresh),
                "/api/v1/auth/logout" => post(handlers::auth_logout::auth_logout),
                // Unreachable while the table stays in sync — see the
                // conformance test in tests/gateway_contract_conformance.rs.
                _ => axum::routing::any(proxy_h::proxy_request),
            },
        );
    }

    // Protected domain routes. The proxy forwarder passes the canonical path
    // verbatim to the upstream — every upstream implements the canonical
    // namespace `/api/v1/<domain>/...` natively, so no rewrite is needed.
    let mut protected = Router::<AppState>::new();
    for spec in routes::protected_routes() {
        protected = protected.route(
            spec.pattern,
            match spec.methods {
                [Method::Get, Method::Patch] => {
                    get(proxy_h::proxy_request).patch(proxy_h::proxy_request)
                }
                [Method::Get] => get(proxy_h::proxy_request),
                [Method::Post] => post(proxy_h::proxy_request),
                _ => axum::routing::any(proxy_h::proxy_request),
            },
        );
    }

    let protected = protected
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

    // All sub-routers below share `AppState`; `.with_state(state)` at the end
    // erases it so the assembled router can be served.
    Router::<AppState>::new()
        .merge(infra)
        // Public auth routes (start/callback are unauthenticated;
        // refresh/logout handle their own auth).
        .merge(auth_routes)
        .merge(protected)
        // Trace-id propagation applies to every route so the id is present
        // on upstream forwards and on the client-visible response.
        .layer(middleware::from_fn(propagate_trace_id))
        // CORS is applied last (outermost) so it also covers the error
        // responses the auth and rate-limit layers produce — a 401 without
        // `Access-Control-Allow-Origin` is unreadable by the browser and
        // surfaces as an opaque network error instead of an auth failure.
        .layer(cors_layer(&state))
        .with_state(state)
}

/// Build the CORS layer from the configured origin allow-list.
///
/// A wildcard is accepted only for local development. `allow_credentials`
/// is never combined with `*`, which the spec forbids and browsers reject,
/// so a wildcard environment drops credentials and the caller falls back to
/// bearer headers rather than cookies.
fn cors_layer(state: &AppState) -> tower_http::cors::CorsLayer {
    use tower_http::cors::{Any, CorsLayer};

    let origins = &state.config().cors_allowed_origins;
    let wildcard = origins.iter().any(|o| o == "*");

    if wildcard {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let parsed: Vec<axum::http::HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
        .collect();
    CorsLayer::new()
        .allow_origin(parsed)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        // `x-trace-id` and `x-member-id` are set by the client/gateway; the
        // client also sends `Idempotency-Key` on mutations.
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("idempotency-key"),
            axum::http::HeaderName::from_static("x-trace-id"),
        ])
        .expose_headers([axum::http::HeaderName::from_static("x-trace-id")])
        .max_age(std::time::Duration::from_secs(600))
}
