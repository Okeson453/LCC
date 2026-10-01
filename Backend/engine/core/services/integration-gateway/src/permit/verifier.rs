//! Permit token verification.
//!
//! Verifies the Ed25519-signed claims issued by the Compliance Governor.
//! Rejects:
//! - bad signature
//! - wrong audience (must be "integration-gateway")
//! - wrong issuer (must be "compliance-governor")
//! - expired (exp ≤ now)
//! - not-yet-valid (iat > now)
//! - wrong `jti` reuse (replay — checked at the dispatcher via Redis)
//! - wrong `sub` (member binding — F-71)
//!
//! The Integration Gateway ONLY holds the public half of the Ed25519 key —
//! it cannot forge permits even if fully compromised (see ADR-0003).
//!
//! ## F-71 — member binding
//!
//! The verifier compares the `sub` claim against the authenticated member
//! ID on the request. If they mismatch, verification fails with
//! `MemberMismatch`. This closes the "permit replayed against a different
//! member_id" attack.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

use chrono::Utc;
use lcc_compliance::permit_token::PermitClaims;
use lcc_compliance::permit_token::PermitVerifier as LccPermitVerifier;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PermitError {
    #[error("permit_token signature/format invalid: {0}")]
    InvalidSignature(String),
    #[error("permit_token expired (now={now}, exp={exp})")]
    Expired { now: i64, exp: i64 },
    #[error("permit_token audience mismatch (expected integration-gateway, got {actual})")]
    AudienceMismatch { actual: String },
    #[error("permit_token issuer mismatch (expected {expected}, got {actual})")]
    IssuerMismatch { expected: String, actual: String },
    #[error("permit_token action_id mismatch (expected {expected}, got {actual})")]
    ActionMismatch { expected: String, actual: String },
    /// F-AUDIT-23: the `typ` claim did not match the action being executed —
    /// a permit minted for one action type was accepted for another.
    #[error("permit_token action_type mismatch (expected {expected}, got {actual})")]
    ActionTypeMismatch { expected: String, actual: String },
    /// F-71: claim `sub` did not match the authenticated member.
    #[error("permit_token member binding failed (claim sub={actual}, request member_id={expected})")]
    MemberMismatch { expected: String, actual: String },
    #[error("permit_token invalid risk_tier: {0}")]
    InvalidRiskTier(String),
    #[error("permit_token from in-the-future (iat={iat}, now={now})")]
    NotYetValid { iat: i64, now: i64 },
    #[error("permit_token key not loaded")]
    KeyNotLoaded,
    /// F-71: a permit-token may be reused. The Gateway tracks `jti` in a
    /// Redis dedupe keyed by `permit:seen:<jti>` for the same TTL window as
    /// the token itself. This variant is returned when a duplicate is detected.
    #[error("permit_token replay detected (jti={0})")]
    Replay(String),
}

#[derive(Debug, Clone)]
pub struct PermitVerifier {
    inner: LccPermitVerifier,
}

impl PermitVerifier {
    /// Build a verifier with the public half of the Governor's Ed25519 key.
    /// Production: the public key is fetched from the audit-svc key registry
    /// (`permit-signing-key` / current `kid`).
    ///
    /// For local dev, the key may be derived from a base64-encoded 32-byte
    /// secret via the `PERMIT_VERIFY_PUBKEY_B64` env var.
    pub fn from_public_key_bytes(public_key_b64: &str) -> Result<Self, PermitError> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes = STANDARD
            .decode(public_key_b64.trim())
            .map_err(|e| PermitError::InvalidSignature(format!("pubkey decode: {e}")))?;
        if bytes.len() != 32 {
            return Err(PermitError::InvalidSignature(format!(
                "pubkey must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&arr)
            .map_err(|e| PermitError::InvalidSignature(format!("pubkey parse: {e}")))?;
        let inner = lcc_compliance::permit_token::public_only_verifier(verifying_key, "integration-gateway");
        Ok(Self { inner })
    }

    /// Verify a permit_token. Returns the claims on success; a typed error on
    /// any failure (loud, never silent).
    ///
    /// `expected_member_id` enforces F-71 — the `sub` claim must equal it.
    ///
    /// F-AUDIT-23: `expected_action_type` is new. The permit's `typ` claim
    /// carries the action type the governor approved, but nothing ever
    /// compared it to the action being executed. A permit minted for a
    /// low-risk `like` was therefore accepted for a `post_publish` or a
    /// `direct_message` — a permit-confusion / privilege-escalation path, since
    /// the whole security model rests on a permit authorising exactly one
    /// action of exactly one type for exactly one member.
    pub fn verify(
        &self,
        token: &str,
        expected_action_id: &Uuid,
        expected_member_id: &Uuid,
        expected_action_type: &str,
    ) -> Result<PermitClaims, PermitError> {
        let claims = self
            .inner
            .verify(token)
            .map_err(|e| PermitError::InvalidSignature(e.to_string()))?;

        // F-71: hard member binding.
        let expected_member_str = expected_member_id.to_string();
        if claims.sub != expected_member_str {
            return Err(PermitError::MemberMismatch {
                expected: expected_member_str,
                actual: claims.sub,
            });
        }

        // Action binding.
        if claims.act != expected_action_id.to_string() {
            return Err(PermitError::ActionMismatch {
                expected: expected_action_id.to_string(),
                actual: claims.act,
            });
        }

        // Action-type binding: a permit is scoped to the action it approved.
        if claims.typ != expected_action_type {
            return Err(PermitError::ActionTypeMismatch {
                expected: expected_action_type.to_string(),
                actual: claims.typ.clone(),
            });
        }

        // Audience + issuer are enforced inside `verify`; we re-assert for clarity.
        if claims.aud != "integration-gateway" {
            return Err(PermitError::AudienceMismatch { actual: claims.aud });
        }
        if claims.iss != "compliance-governor" {
            return Err(PermitError::IssuerMismatch {
                expected: "compliance-governor".into(),
                actual: claims.iss,
            });
        }

        let now = Utc::now().timestamp();
        if claims.exp <= now {
            return Err(PermitError::Expired { now, exp: claims.exp });
        }
        if claims.iat > now {
            return Err(PermitError::NotYetValid { iat: claims.iat, now });
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use lcc_compliance::permit_token::{signing_only_signer, PermitSigner};
    use rand::rngs::OsRng;

    fn verifier_and_signer() -> (PermitVerifier, PermitSigner, SigningKey) {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let vk = sk.verifying_key();
        let vk_b64 = base64::engine::general_purpose::STANDARD.encode(vk.to_bytes());
        let v = PermitVerifier::from_public_key_bytes(&vk_b64).expect("verifier");
        let iss = signing_only_signer(sk.clone(), "integration-gateway");
        (v, iss, sk)
    }

    fn make_claims(iss: &PermitSigner, member: Uuid, action: Uuid, ttl: i64) -> String {
        use lcc_compliance::permit_token::PermitClaimsBuilder;
        let claims = PermitClaimsBuilder::new(
            member.to_string(),
            action.to_string(),
            "like",
            "tier_2_light_engagement",
            "ccfg-active",
        )
        .approval_id(Some(Uuid::now_v7().to_string()))
        .ttl_seconds(ttl)
        .build();
        iss.issue(&claims).expect("issue")
    }

    #[test]
    fn valid_token_round_trips() {
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let action = Uuid::now_v7();
        let token = make_claims(&iss, member, action, 60);
        let claims = v.verify(&token, &action, &member, "like").expect("verify");
        assert_eq!(claims.act, action.to_string());
        assert_eq!(claims.sub, member.to_string());
    }

    #[test]
    fn action_mismatch_rejected() {
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let action = Uuid::now_v7();
        let token = make_claims(&iss, member, action, 60);
        let wrong = Uuid::now_v7();
        let err = v.verify(&token, &wrong, &member, "like").expect_err("must reject");
        assert!(matches!(err, PermitError::ActionMismatch { .. }));
    }

    #[test]
    fn member_mismatch_rejected_f71() {
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let impostor = Uuid::now_v7();
        let action = Uuid::now_v7();
        let token = make_claims(&iss, member, action, 60);
        let err = v.verify(&token, &action, &impostor, "like").expect_err("must reject");
        assert!(matches!(err, PermitError::MemberMismatch { .. }));
    }

    #[test]
    fn expired_token_rejected() {
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let action = Uuid::now_v7();
        // ttl ≤ 0 produces an already-expired token.
        let token = make_claims(&iss, member, action, 0);
        let err = v.verify(&token, &action, &member, "like").expect_err("must reject");
        assert!(matches!(err, PermitError::Expired { .. }));
    }

    #[test]
    fn action_type_mismatch_rejected_f_audit_23() {
        // A permit minted for a low-risk `like` must not authorise a
        // `post_publish` or a `direct_message`.
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let action = Uuid::now_v7();
        let token = make_claims(&iss, member, action, 60);
        let err = v
            .verify(&token, &action, &member, "post_publish")
            .expect_err("must reject");
        assert!(matches!(err, PermitError::ActionTypeMismatch { .. }));
    }

    #[test]
    fn tampered_signature_rejected() {
        let (v, iss, _) = verifier_and_signer();
        let member = Uuid::now_v7();
        let action = Uuid::now_v7();
        let token = make_claims(&iss, member, action, 60);
        // Flip the last char of the signature segment.
        let mut tampered = token.clone();
        let last = tampered.pop().unwrap();
        let flipped = if last == 'A' { 'B' } else { 'A' };
        tampered.push(flipped);
        let err = v.verify(&tampered, &action, &member, "like").expect_err("must reject");
        assert!(matches!(err, PermitError::InvalidSignature(_)));
    }
}
