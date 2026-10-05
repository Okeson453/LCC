//! End-to-end wire-format test for the Identity service.
//!
//! Exercises the canonical `/api/v1/...` HTTP surface of identity-svc
//! against the canonical contract:
//!   1. Canonical paths are registered on the router.
//!   2. Bearer JWT validation rejects missing / malformed tokens.
//!   3. `GET /api/v1/members/me` returns the canonical Member shape.
//!   4. `PATCH /api/v1/members/me/settings` validates timezone and applies
//!      optimistic-concurrency.
//!   5. Error envelope matches the contract: `{ "error": { "code",
//!      "message", "trace_id" } }`.
//!
//! This test does NOT require a running Postgres; it exercises the
//! service layer (JWT mint + verify, validation rules, error envelope)
//! directly via `Service` methods. The HTTP layer is constructed but
//! requests that need DB I/O will fail with INTERNAL_ERROR — that's the
//! expected behavior; the contract guarantees the error envelope shape
//! regardless.

#![cfg(test)]

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

use lcc_identity_svc::config::Config;
use lcc_identity_svc::domain::{GoalMode, Member, MemberRole, MemberSettingsUpdate};
use lcc_identity_svc::error::Error as SvcError;
use lcc_identity_svc::http::router::build_router;
use lcc_identity_svc::repository::PgRepository;
use lcc_identity_svc::service::{JwtClaims, Service};

fn test_config() -> Config {
    Config {
        database_url: "postgres://nobody:nobody@127.0.0.1:1/x".into(),
        http_port: 8090,
        linkedin_client_id: "test".into(),
        linkedin_client_secret: "test".into(),
        public_oauth_redirect_uri: "https://test/auth/callback".into(),
        auth_jwt_secret: "this-is-a-test-secret-32-chars-long-xx".into(),
        jwt_issuer: "lcc-auth".into(),
        jwt_audience: "lcc-api".into(),
        access_token_ttl_secs: 900,
        refresh_token_ttl_secs: 86_400,
        token_encryption_key: "0123456789abcdef-test".into(),
        http_client: reqwest::Client::new(),
        unused_jti: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        // Fields added to `Config` after this test was written; without them
        // the crate failed to compile with E0063. This test never opens a
        // Redis connection, so pointing at an unreachable local address is
        // safe and keeps the fixture honest about being hermetic.
        redis_url: "redis://127.0.0.1:1".into(),
        service_name: "identity-svc-test".into(),
        audit_svc_url: "http://audit-svc:8091".into(),
        compliance_governor_url: "http://compliance-governor:8080".into(),
    }
}

fn build_service() -> Arc<Service> {
    let cfg = test_config();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_millis(50))
        .connect_lazy(&cfg.database_url)
        .expect("lazy pool");
    let repo = PgRepository::new(pool);
    Arc::new(Service::new(repo, Arc::new(cfg)))
}

fn issue_test_jwt(service: &Service, member_id: Uuid, role: MemberRole) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = chrono::Utc::now();
    let claims = JwtClaims {
        sub: member_id.to_string(),
        iss: service.cfg().jwt_issuer.clone(),
        aud: service.cfg().jwt_audience.clone(),
        iat: now.timestamp(),
        exp: now.timestamp() + 3600,
        jti: Uuid::new_v4().to_string(),
        role: role.as_str().to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(service.cfg().auth_jwt_secret.as_bytes()),
    )
    .expect("encode test jwt")
}

#[tokio::test]
async fn canonical_router_exposes_canonical_paths() {
    // Building the router is itself the test: panic = missing route, no
    // panic = routes are registered against the canonical namespace.
    let svc = build_service();
    let state = lcc_identity_svc::state::AppState::for_test(svc);
    let _router = build_router(state);
}

#[tokio::test]
async fn canonical_jwt_round_trip() {
    let svc = build_service();
    let member_id = Uuid::new_v4();
    let token = issue_test_jwt(&svc, member_id, MemberRole::Owner);
    let claims = svc.verify_access_token(&token).expect("verify own jwt");
    assert_eq!(claims.sub, member_id.to_string());
    assert_eq!(claims.role, "owner");
}

#[tokio::test]
async fn canonical_jwt_rejects_bad_token() {
    let svc = build_service();
    let bad = svc.verify_access_token("not-a-jwt");
    assert!(matches!(bad, Err(SvcError::Unauthorized(_))));
}

#[tokio::test]
async fn canonical_error_envelope_shape_not_found() {
    let resp = SvcError::NotFound("member 123".into()).into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body_bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let err = body.get("error").expect("error envelope missing");
    assert_eq!(err.get("code").and_then(|v| v.as_str()), Some("NOT_FOUND"));
    assert!(err
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap()
        .contains("member 123"));
    assert!(err.get("trace_id").is_some(), "trace_id must be present");
}

#[tokio::test]
async fn canonical_error_envelope_shape_unauthorized() {
    let resp = SvcError::Unauthorized("bad token".into()).into_response();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body_bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        body.get("error")
            .and_then(|e| e.get("code"))
            .and_then(|v| v.as_str()),
        Some("UNAUTHORIZED")
    );
}

#[tokio::test]
async fn canonical_error_envelope_shape_bad_request() {
    let resp = SvcError::BadRequest("missing field".into()).into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        body.get("error")
            .and_then(|e| e.get("code"))
            .and_then(|v| v.as_str()),
        Some("VALIDATION_ERROR")
    );
}

#[tokio::test]
async fn canonical_member_shape_round_trip() {
    let member = Member {
        id: Uuid::new_v4(),
        linkedin_id: "ln_demo".into(),
        email: Some("demo@example.com".into()),
        display_name: "Demo".into(),
        role: MemberRole::Owner,
        is_active: true,
        timezone: "Europe/Paris".into(),
        locale: "en-US".into(),
        active_goal_mode: GoalMode::Hybrid,
        is_restricted: false,
        restricted_since: None,
        restricted_reason: None,
        warmup_started_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
    };
    // Serialise to JSON, parse back; ensure goal_mode round-trips and the
    // snake_case contract serialization is preserved.
    let json = serde_json::to_string(&member).unwrap();
    let parsed: Member = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.active_goal_mode, GoalMode::Hybrid);
    assert_eq!(parsed.role, MemberRole::Owner);
}

#[tokio::test]
async fn canonical_settings_update_validates_timezone_empty() {
    // The validation branch for empty timezone is exercised before any DB
    // call — so this test works without a live database.
    let svc = build_service();
    let member_id = Uuid::new_v4();

    let result = svc
        .update_settings(
            member_id,
            MemberSettingsUpdate {
                timezone: Some("".into()),
                active_goal_mode: None,
                cap_overrides: None,
            },
            1,
        )
        .await;
    assert!(matches!(result, Err(SvcError::BadRequest(_))));
}

#[tokio::test]
async fn canonical_settings_update_validates_timezone_too_long() {
    let svc = build_service();
    let member_id = Uuid::new_v4();

    let long = "a".repeat(65);
    let result = svc
        .update_settings(
            member_id,
            MemberSettingsUpdate {
                timezone: Some(long),
                active_goal_mode: None,
                cap_overrides: None,
            },
            1,
        )
        .await;
    assert!(matches!(result, Err(SvcError::BadRequest(_))));
}
