//! Tests for the auth crate (JWT, RBAC).

use lcc_auth::jwt::{decode_token, encode_token, Claims};
use lcc_auth::rbac::Role;

#[test]
fn jwt_round_trip() {
    let claims = Claims {
        sub: "user-1".into(),
        iss: "identity-svc".into(),
        aud: "lcc-api".into(),
        exp: chrono::Utc::now().timestamp() + 60,
        iat: chrono::Utc::now().timestamp(),
        roles: vec![Role::Member],
    };
    let secret = b"test-secret-key-for-jwt-32-bytes!!";
    let token = encode_token(&claims, secret).expect("encode");
    let parsed = decode_token(&token, secret, "lcc-api").expect("decode");
    assert_eq!(parsed.sub, "user-1");
    assert_eq!(parsed.roles, vec![Role::Member]);
}

#[test]
fn jwt_wrong_audience_fails() {
    let claims = Claims {
        sub: "user-1".into(),
        iss: "identity-svc".into(),
        aud: "lcc-api".into(),
        exp: chrono::Utc::now().timestamp() + 60,
        iat: chrono::Utc::now().timestamp(),
        roles: vec![Role::Member],
    };
    let secret = b"test-secret-key-for-jwt-32-bytes!!";
    let token = encode_token(&claims, secret).expect("encode");
    let result = decode_token(&token, secret, "different-audience");
    assert!(result.is_err());
}

#[test]
fn jwt_expired_fails() {
    let claims = Claims {
        sub: "user-1".into(),
        iss: "identity-svc".into(),
        aud: "lcc-api".into(),
        exp: chrono::Utc::now().timestamp() - 1,
        iat: chrono::Utc::now().timestamp() - 60,
        roles: vec![Role::Member],
    };
    let secret = b"test-secret-key-for-jwt-32-bytes!!";
    let token = encode_token(&claims, secret).expect("encode");
    let result = decode_token(&token, secret, "lcc-api");
    assert!(result.is_err());
}

#[test]
fn role_serde() {
    let r = Role::ComplianceReviewer;
    let s = serde_json::to_string(&r).expect("serialize");
    let parsed: Role = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(parsed, Role::ComplianceReviewer);
}
