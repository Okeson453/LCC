//! Axum middleware that enforces authentication (and optionally a permission)
//! for an entire route subtree.
//!
//! # Why middleware rather than per-handler checks
//!
//! F-AUDIT-60: `compliance-governor`'s `/api/v1/admin/compliance/**` routes
//! enforce authorization at the gateway but perform **no** service-level check
//! of their own. Every one of those handlers is a state change on the
//! compliance configuration that governs what the platform is allowed to send —
//! the highest-consequence surface in the system. A caller reaching the
//! service directly, bypassing the gateway, could propose, review and activate
//! a config version with no credential at all.
//!
//! Per-handler checks are exactly the pattern that produced the original gap:
//! a new handler is added, the check is forgotten, and the route is open.
//! A `route_layer` over the subtree makes the secure behaviour the default —
//! a handler added under it is protected without anyone remembering to do
//! anything.
//!
//! Layering it at the router means the check cannot be bypassed by an
//! alternative execution path to the handler, which is the property the design
//! asks for ("security controls cannot be bypassed through direct service
//! access").
//!
//! # Verifier injection
//!
//! The middleware takes a `JwtVerifier` rather than reading
//! `LCC_AUTH_JWT_SECRET` per request. That is both a performance property (the
//! decoding key is parsed once at startup) and a testability one: the secret
//! becomes an explicit input instead of process-global state that concurrent
//! tests would race on.

use std::sync::Arc;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use lcc_error::LccError;

use crate::identity::Caller;
use crate::jwt::JwtVerifier;
use crate::rbac::Permission;

/// A shared, pre-built verifier. Cheap to clone (it wraps an `Arc` internally).
pub type SharedVerifier = Arc<JwtVerifier>;

/// Verifies the bearer token and inserts the [`Caller`] into extensions.
///
/// Rejection is 401 for any unusable credential: missing header, wrong scheme,
/// bad signature, expired, non-UUID `sub`, or an unrecognised role.
pub async fn require_auth(verifier: SharedVerifier, request: Request, next: Next) -> Response {
    match verify_caller(&verifier, &request) {
        Ok(caller) => {
            let mut request = request;
            request.extensions_mut().insert(caller);
            next.run(request).await
        }
        Err(e) => error_response(e),
    }
}

/// Verifies the bearer token, then requires `perm`.
///
/// Rejection is 403 (Forbidden) when the caller is authenticated but lacks the
/// permission, and 401 (Unauthorized) when there is no usable credential —
/// keeping the two cases distinguishable, as the design requires.
pub async fn require_permission(
    verifier: SharedVerifier,
    perm: Permission,
    request: Request,
    next: Next,
) -> Response {
    match verify_caller(&verifier, &request) {
        Ok(caller) => match caller.require(perm) {
            Ok(()) => {
                let mut request = request;
                request.extensions_mut().insert(caller);
                next.run(request).await
            }
            Err(e) => error_response(e),
        },
        Err(e) => error_response(e),
    }
}

fn verify_caller(verifier: &JwtVerifier, request: &Request) -> Result<Caller, LccError> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| LccError::Unauthorized("missing bearer token".into()))?;

    let claims = verifier
        .verify(token)
        .map_err(|e| LccError::Unauthorized(format!("token rejected: {e}")))?;

    let member_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| LccError::Unauthorized("sub is not a uuid".into()))?;
    // An unrecognised role is rejected rather than defaulted to a
    // least-privilege role: defaulting invites accidentally widening the
    // default later, whereas rejecting means a newly-added role cannot be
    // silently under-privileged across the whole estate.
    let role: crate::rbac::Role = claims
        .role
        .parse()
        .map_err(|_| LccError::Unauthorized("unknown role in token".into()))?;

    Ok(Caller { member_id, role })
}

fn error_response(e: LccError) -> Response {
    let status = match e {
        LccError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        LccError::Forbidden(_) => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    // Deliberately does not echo the reason: token-rejection detail can help an
    // attacker distinguish "bad signature" from "expired" from "unknown role".
    let body = match status {
        StatusCode::UNAUTHORIZED => serde_json::json!({"error": "unauthorized"}),
        StatusCode::FORBIDDEN => serde_json::json!({"error": "forbidden"}),
        _ => serde_json::json!({"error": "internal error"}),
    };
    (status, axum::Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    // F-AUDIT-51: the workspace denies `clippy::unwrap_used` / `expect_used`
    // because an `unwrap` on a `Result` can take a production service down.
    // In a test binary the opposite holds — panicking is the failure signal.
    // Scoped to this test module so the production lints stay intact.
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::jwt::JwtIssuer;
    use axum::{body::Body, routing::get, Router};
    use tower::ServiceExt as _;
    use uuid::Uuid;

    const SECRET: &[u8] = b"jwt-secret-test";

    fn verifier() -> SharedVerifier {
        Arc::new(JwtVerifier::new(SECRET, "lcc.api-gateway", "lcc-api"))
    }

    fn token_for(role: &str) -> String {
        let iss = JwtIssuer::new(SECRET, "k1", "lcc.api-gateway", "lcc-api", 15);
        iss.issue(&Uuid::now_v7(), role).expect("mint").0
    }

    /// Mirrors the real compliance-governor wiring: an admin subtree guarded by
    /// `ManageComplianceConfig`, plus an unguarded health route.
    fn admin_router() -> Router {
        let v = verifier();
        Router::new()
            .route(
                "/api/v1/admin/compliance/config-versions",
                get(|| async { "ok" }),
            )
            .route_layer(axum::middleware::from_fn(
                move |request: axum::http::Request<axum::body::Body>, next: Next| {
                    let v = v.clone();
                    async move {
                        require_permission(v, Permission::ManageComplianceConfig, request, next)
                            .await
                    }
                },
            ))
            .route("/healthz", get(|| async { "ok" }))
    }

    async fn status(app: &Router, uri: &str, auth: Option<&str>) -> StatusCode {
        let mut b = axum::http::Request::builder().uri(uri);
        if let Some(t) = auth {
            b = b.header("authorization", format!("Bearer {t}"));
        }
        app.clone()
            .oneshot(b.body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    const ADMIN_URI: &str = "/api/v1/admin/compliance/config-versions";

    #[tokio::test]
    async fn admin_route_rejects_missing_token() {
        assert_eq!(
            status(&admin_router(), ADMIN_URI, None).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn admin_route_rejects_garbage_token() {
        assert_eq!(
            status(&admin_router(), ADMIN_URI, Some("obviously-not-a-jwt")).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn admin_route_rejects_token_signed_with_another_secret() {
        let other = JwtIssuer::new(b"attacker-secret", "k1", "lcc.api-gateway", "lcc-api", 15);
        let forged = other.issue(&Uuid::now_v7(), "owner").unwrap().0;
        assert_eq!(
            status(&admin_router(), ADMIN_URI, Some(&forged)).await,
            StatusCode::UNAUTHORIZED,
            "a token signed with a different secret must never be accepted"
        );
    }

    #[tokio::test]
    async fn admin_route_allows_owner() {
        let t = token_for("owner");
        assert_eq!(
            status(&admin_router(), ADMIN_URI, Some(&t)).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn admin_route_allows_admin() {
        let t = token_for("admin");
        assert_eq!(
            status(&admin_router(), ADMIN_URI, Some(&t)).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn admin_route_forbids_unprivileged_roles() {
        // 403, not 401: the caller is authenticated, they just may not do this.
        for role in ["assistant", "reviewer", "auditor"] {
            let t = token_for(role);
            assert_eq!(
                status(&admin_router(), ADMIN_URI, Some(&t)).await,
                StatusCode::FORBIDDEN,
                "{role} must be forbidden from managing compliance config"
            );
        }
    }

    #[tokio::test]
    async fn health_route_is_not_gated() {
        assert_eq!(
            status(&admin_router(), "/healthz", None).await,
            StatusCode::OK
        );
    }
}
