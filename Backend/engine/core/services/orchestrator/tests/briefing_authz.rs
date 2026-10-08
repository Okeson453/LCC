//! Live-database integration tests for the orchestrator's member-scoped routes.
//!
//! # These tests do not skip
//!
//! They need PostgreSQL and are skipped only when `LCC_TEST_DATABASE_URL` is
//! unset, which is reported loudly rather than passing silently.
//!
//! The subject is the cross-tenant read fixed in this pass. The briefing
//! handlers used to take `member_id` straight from the path and never looked at
//! the `Authorization` header, so any caller who could reach the service could
//! read any member's daily briefing by editing one path segment. These tests
//! drive the real router, so they fail if the gate is ever bypassed -- including
//! by a route added later that forgets to be mounted inside it.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use lcc_auth::JwtIssuer;
use sqlx::PgPool;
use uuid::Uuid;

const SECRET: &[u8] = b"orchestrator-authz-test-secret-32b";

async fn test_pool(test: &str) -> Option<PgPool> {
    let url = std::env::var("LCC_TEST_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url)
        .await
        .unwrap_or_else(|e| panic!("{test}: cannot reach LCC_TEST_DATABASE_URL: {e}"));
    Some(pool)
}

fn issuer() -> JwtIssuer {
    JwtIssuer::new(
        SECRET,
        "test-key",
        lcc_auth::DEFAULT_JWT_ISSUER,
        lcc_auth::DEFAULT_JWT_AUDIENCE,
        10,
    )
}

fn verifier() -> lcc_auth::SharedVerifier {
    Arc::new(lcc_auth::JwtVerifier::new(
        SECRET,
        lcc_auth::DEFAULT_JWT_ISSUER,
        lcc_auth::DEFAULT_JWT_AUDIENCE,
    ))
}

async fn get_briefing(app: axum::Router, member: Uuid, token: Option<&str>) -> StatusCode {
    let mut req = Request::builder()
        .uri(format!("/api/v1/members/{member}/briefing/today"))
        .method("GET");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let res = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    res.status()
}

/// An unauthenticated caller must not be able to read a briefing. This is the
/// case that was wide open before: the handler never touched the header.
#[tokio::test]
async fn briefing_without_a_token_is_unauthorized() {
    let Some(pool) = test_pool("briefing_without_a_token_is_unauthorized").await else {
        eprintln!("SKIPPED: LCC_TEST_DATABASE_URL not set");
        return;
    };
    let state = lcc_orchestrator::state::AppState::new(pool, redis()).await;
    let app = lcc_orchestrator::http::build_router(state, verifier());

    let victim = Uuid::new_v4();
    assert_eq!(
        get_briefing(app, victim, None).await,
        StatusCode::UNAUTHORIZED,
        "an unauthenticated request must not read any member's briefing"
    );
}

/// The core regression test. A valid token for member A must not unlock member
/// B's briefing, and the refusal must be 404 rather than 403: a 403 would
/// confirm that member B exists, which is the existence oracle the security
/// design forbids.
#[tokio::test]
async fn one_member_cannot_read_another_members_briefing() {
    let Some(pool) = test_pool("one_member_cannot_read_another_members_briefing").await else {
        eprintln!("SKIPPED: LCC_TEST_DATABASE_URL not set");
        return;
    };
    let state = lcc_orchestrator::state::AppState::new(pool, redis()).await;
    let app = lcc_orchestrator::http::build_router(state, verifier());

    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();
    let (alice_token, _) = issuer().issue(&alice, "owner").expect("issue alice");

    assert_eq!(
        get_briefing(app.clone(), bob, Some(&alice_token)).await,
        StatusCode::NOT_FOUND,
        "member A must not reach member B's briefing, and the refusal must be 404"
    );

    // And alice's own briefing is not blocked by the same rule, so the 404
    // above is the member mismatch and not a blanket denial.
    let own = get_briefing(app, alice, Some(&alice_token)).await;
    assert_ne!(
        own,
        StatusCode::UNAUTHORIZED,
        "alice's own token must authenticate"
    );
    assert_ne!(
        own,
        StatusCode::NOT_FOUND,
        "alice must be able to read her own briefing"
    );
}

/// A garbage bearer token is 401, not 403: the request was not authenticated
/// at all, which is a different condition from "authenticated but not allowed".
#[tokio::test]
async fn briefing_with_a_forged_token_is_unauthorized() {
    let Some(pool) = test_pool("briefing_with_a_forged_token_is_unauthorized").await else {
        eprintln!("SKIPPED: LCC_TEST_DATABASE_URL not set");
        return;
    };
    let state = lcc_orchestrator::state::AppState::new(pool, redis()).await;
    let app = lcc_orchestrator::http::build_router(state, verifier());

    assert_eq!(
        get_briefing(app, Uuid::new_v4(), Some("not.a.jwt")).await,
        StatusCode::UNAUTHORIZED
    );
}

/// The health probes sit outside the auth layer on purpose -- the kubelet has no
/// bearer token -- so this guards against a future edit that closes the service
/// off from its own liveness probe.
#[tokio::test]
async fn health_probes_stay_unauthenticated() {
    let Some(pool) = test_pool("health_probes_stay_unauthenticated").await else {
        eprintln!("SKIPPED: LCC_TEST_DATABASE_URL not set");
        return;
    };
    let state = lcc_orchestrator::state::AppState::new(pool, redis()).await;
    let app = lcc_orchestrator::http::build_router(state, verifier());

    for path in ["/healthz", "/readyz"] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_ne!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "{path} must stay reachable without a token"
        );
    }
}

/// Minimal Redis pool for handler construction. No command is issued on the
/// paths under test, so an unreachable Redis must not change the outcome.
fn redis() -> deadpool_redis::Pool {
    deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("redis pool config")
}

use tower::ServiceExt as _;
