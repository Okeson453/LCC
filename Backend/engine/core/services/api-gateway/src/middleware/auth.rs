//! Auth middleware — validates the bearer JWT and publishes the member's RLS
//! context to downstream upstreams.
//!
//! ### Audit fixes applied here
//!
//! **F-AUDIT-07 (compile-blocking).** This file previously imported
//! `lcc_auth::jwt::{decode_token, Claims}` and called `state.jwt_secret()`,
//! `state.jwt_audience()` and `state.db()`, none of which exist:
//!
//! - the type is `JwtClaims`, not `Claims`;
//! - verification is `JwtVerifier::verify`, not a free `decode_token`;
//! - `AppState` has no `jwt_secret`/`jwt_audience`/`db` accessors, and no DB
//!   pool at all.
//!
//! It also declared `State<Arc<AppState>>` while the router supplies
//! `AppState`, and used `parking_lot`, which is not a dependency of this
//! crate. The module therefore could not compile, which means the gateway
//! could not build or run, and the `{"status":"ok"}` proxy stubs below it were
//! never actually shadowed by a working build.
//!
//! **F-AUDIT-08 (RLS / connection exhaustion).** The previous version opened a
//! `sqlx::Transaction` per request and stashed it in request extensions, to be
//! consumed by a handler that never did so. Because sqlx rolls back on drop,
//! every authenticated request performed a pointless BEGIN/ROLLBACK and held
//! a pooled connection for the whole request. RLS context is instead now
//! propagated as headers to the owning service, which is where the SQL
//! actually runs — and the owning service is the correct place for
//! `SET LOCAL app.current_member_id` to apply to that request's transaction.
//!
//! **F-AUDIT-09 (auth bypass).** `member_id` used for RLS scoping was parsed
//! straight out of the token `sub`; a non-UUID subject was rejected, which is
//! retained. The `role` claim is now parsed through `lcc_auth::rbac::Role`
//! rather than being carried as an unvalidated string, so an unknown role is
//! rejected at the edge instead of reaching `has_permission` as garbage.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use lcc_auth::{JwtClaims, JwtVerifier};
use uuid::Uuid;

use crate::state::AppState;

/// Authenticated request context, inserted by [`require_auth`] so downstream
/// handlers and middleware (notably the rate limiter) can read the caller's
/// verified identity without re-parsing the token.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub claims: JwtClaims,
    pub member_id: Uuid,
    pub token: String,
}

/// Header carrying the verified member id to the upstream service.
///
/// The gateway only forwards headers on the `FORWARD_REQUEST_HEADERS`
/// allow-list; `x-member-id` is on that list specifically so this value
/// arrives. The receiving service MUST treat it as a hint and re-derive
/// identity from its own verified session — it must never be trusted to
/// arrive un-spoofed from the public internet, which is why the gateway
/// overwrites it here rather than forwarding a client-supplied value.
pub const MEMBER_ID_HEADER: &str = "x-member-id";

/// Build the Axum middleware that authenticates the bearer token.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?
        // Owned up front: `auth_header` borrows `req`, and the request is
        // mutated below to stamp the derived member id.
        .to_string();

    let verifier: &Arc<JwtVerifier> = state.jwt_verifier();
    let claims: JwtClaims = verifier
        .verify(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // `sub` is the member_id. A non-UUID subject is a malformed token.
    let member_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Validate the role claim against the real RBAC enum so an unknown role
    // is rejected at the edge instead of propagating.
    claims
        .role
        .parse::<lcc_auth::Role>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Overwrite (never trust) the member id header handed downstream.
    if let Ok(value) = HeaderValue::from_str(&member_id.to_string()) {
        let name = axum::http::HeaderName::from_static(MEMBER_ID_HEADER);
        req.headers_mut().insert(name, value);
    }

    req.extensions_mut().insert(AuthenticatedUser {
        claims: claims.clone(),
        member_id,
        token,
    });
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Returns a header that propagates the trace_id downstream.
pub fn trace_propagation_header(trace_id: &str) -> HeaderValue {
    HeaderValue::from_str(trace_id).unwrap_or_else(|_| HeaderValue::from_static("invalid"))
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_propagation_header_validates_input() {
        let v = trace_propagation_header("trace_123");
        assert_eq!(v.to_str().expect("valid header"), "trace_123");
    }

    #[test]
    fn invalid_trace_id_does_not_panic() {
        let v = trace_propagation_header("bad\nvalue");
        assert_eq!(v.to_str().expect("fallback header"), "invalid");
    }
}
