//! Tests for the auth crate (JWT issuance/verification, RBAC).
//!
//! Written against the real `lcc_auth` surface: `JwtIssuer` / `JwtVerifier` /
//! `JwtClaims` and the five-role `Role` enum. The previous version imported a
//! free `encode_token` / `decode_token` / `Claims` API and a `Role::Member` /
//! `Role::ComplianceReviewer` that do not exist, so this test target never
//! compiled.

use lcc_auth::jwt::{JwtIssuer, JwtVerifier};
use lcc_auth::rbac::{has_permission, Permission, Role};
use uuid::Uuid;

const SECRET: &[u8] = b"test-secret-key-for-jwt-32-bytes!!";
const ISSUER: &str = "lcc-identity-svc";
const AUDIENCE: &str = "lcc-api";

fn issuer() -> JwtIssuer {
    JwtIssuer::new(SECRET, "k1", ISSUER, AUDIENCE, 15)
}

fn verifier() -> JwtVerifier {
    JwtVerifier::new(SECRET, ISSUER, AUDIENCE)
}

#[test]
fn jwt_round_trip() {
    let member_id = Uuid::now_v7();
    let (token, issued) = issuer().issue(&member_id, "owner").expect("issue");
    let parsed = verifier().verify(&token).expect("verify");

    assert_eq!(parsed.sub, member_id.to_string());
    assert_eq!(parsed.role, "owner");
    assert_eq!(parsed.iss, ISSUER);
    assert_eq!(parsed.aud, AUDIENCE);
    // The two ids must be distinct and non-empty so sessions can be tracked.
    assert!(!parsed.jti.is_empty());
    assert!(!parsed.session_id.is_empty());
    assert_ne!(parsed.jti, parsed.session_id);
    assert_eq!(issued.jti, parsed.jti);
}

#[test]
fn each_issue_gets_a_fresh_session() {
    let id = Uuid::now_v7();
    let (_, a) = issuer().issue(&id, "owner").expect("issue");
    let (_, b) = issuer().issue(&id, "owner").expect("issue");
    assert_ne!(a.session_id, b.session_id, "re-login must mint a new session");
    assert_ne!(a.jti, b.jti, "each token needs its own jti");
}

#[test]
fn jwt_wrong_audience_fails() {
    let (token, _) = issuer().issue(&Uuid::now_v7(), "owner").expect("issue");
    let other = JwtVerifier::new(SECRET, ISSUER, "different-audience");
    assert!(other.verify(&token).is_err());
}

#[test]
fn jwt_wrong_issuer_fails() {
    let (token, _) = issuer().issue(&Uuid::now_v7(), "owner").expect("issue");
    let other = JwtVerifier::new(SECRET, "someone-else", AUDIENCE);
    assert!(other.verify(&token).is_err());
}

#[test]
fn jwt_wrong_secret_fails() {
    let (token, _) = issuer().issue(&Uuid::now_v7(), "owner").expect("issue");
    let other = JwtVerifier::new(b"a-completely-different-secret-value", ISSUER, AUDIENCE);
    assert!(other.verify(&token).is_err());
}

#[test]
fn malformed_token_fails() {
    assert!(verifier().verify("not-a-jwt").is_err());
    assert!(verifier().verify("").is_err());
    // A signature from a different key must not verify.
    let other_issuer = JwtIssuer::new(b"a-completely-different-secret-value", "k1", ISSUER, AUDIENCE, 15);
    let (token, _) = other_issuer.issue(&Uuid::now_v7(), "owner").expect("issue");
    assert!(verifier().verify(&token).is_err());
}

#[test]
fn expired_token_fails() {
    // The issuer always stamps `now + ttl`, so mint one with a zero/negative
    // window rather than sleeping.
    let expired = JwtIssuer::new(SECRET, "k1", ISSUER, AUDIENCE, -1);
    let (token, _) = expired.issue(&Uuid::now_v7(), "owner").expect("issue");
    assert!(verifier().verify(&token).is_err());
}

#[test]
fn role_serde_round_trip() {
    for r in [
        Role::Owner,
        Role::Assistant,
        Role::Reviewer,
        Role::Admin,
        Role::Auditor,
    ] {
        let s = serde_json::to_string(&r).expect("serialize");
        let parsed: Role = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(parsed, r);
    }
}

#[test]
fn unknown_role_string_is_rejected() {
    assert!("compliance_reviewer".parse::<Role>().is_err());
    assert!("owner".parse::<Role>().is_ok());
}

#[test]
fn assistant_cannot_approve_sends() {
    assert!(!has_permission(Role::Assistant, Permission::ApproveSend));
    assert!(has_permission(Role::Owner, Permission::ApproveSend));
}

#[test]
fn auditor_is_read_only() {
    assert!(has_permission(Role::Auditor, Permission::ViewAuditLog));
    assert!(!has_permission(Role::Auditor, Permission::ApproveSend));
    assert!(!has_permission(Role::Auditor, Permission::DraftContent));
}
