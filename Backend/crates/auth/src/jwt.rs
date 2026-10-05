//! JWT issuance + verification for the LCC session tokens.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Standard LCC JWT claims.
///
/// Issued by api-gateway after OAuth2/PKCE callback success. Carries the
/// `member_id` for RLS scoping, the role for RBAC, and standard JWT fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtClaims {
    pub iss: String,  // "lcc.api-gateway"
    pub aud: String,  // "lcc-api"
    pub sub: String,  // member_id (UUIDv7 string)
    pub role: String, // "owner"|"assistant"|"reviewer"|"admin"|"auditor"
    pub iat: i64,
    pub exp: i64,
    pub jti: String,        // UUIDv7
    pub session_id: String, // UUIDv7 — one per login
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("jwt encode/decode error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("token expired (now={now}, exp={exp})")]
    Expired { now: i64, exp: i64 },
    #[error("audience mismatch (expected {expected}, got {actual})")]
    AudienceMismatch { expected: String, actual: String },
    #[error("issuer mismatch")]
    IssuerMismatch,
    #[error("role '{0}' unknown")]
    UnknownRole(String),
}

/// Tolerance for clock skew between the issuer and the verifier, in seconds.
///
/// A session token is rejected once it is this far past its `exp`. 30s is
/// enough to absorb normal NTP drift between pods without materially extending
/// a session's life.
pub const JWT_CLOCK_SKEW_SECS: u64 = 30;

pub struct JwtIssuer {
    signing_key: Arc<EncodingKey>,
    key_id: String,
    issuer: String,
    audience: String,
    ttl_minutes: i64,
}

impl JwtIssuer {
    pub fn new(
        secret: &[u8],
        key_id: impl Into<String>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        ttl_minutes: i64,
    ) -> Self {
        Self {
            signing_key: Arc::new(EncodingKey::from_secret(secret)),
            key_id: key_id.into(),
            issuer: issuer.into(),
            audience: audience.into(),
            ttl_minutes,
        }
    }

    pub fn issue(&self, member_id: &Uuid, role: &str) -> Result<(String, JwtClaims), JwtError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.ttl_minutes);
        let jti = Uuid::now_v7();
        let session_id = Uuid::now_v7();

        let claims = JwtClaims {
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            sub: member_id.to_string(),
            role: role.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            jti: jti.to_string(),
            session_id: session_id.to_string(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(self.key_id.clone());

        let token = encode(&header, &claims, &self.signing_key)?;
        Ok((token, claims))
    }
}

pub struct JwtVerifier {
    decoding_key: Arc<DecodingKey>,
    expected_issuer: String,
    expected_audience: String,
}

impl JwtVerifier {
    pub fn new(
        secret: &[u8],
        expected_issuer: impl Into<String>,
        expected_audience: impl Into<String>,
    ) -> Self {
        Self {
            decoding_key: Arc::new(DecodingKey::from_secret(secret)),
            expected_issuer: expected_issuer.into(),
            expected_audience: expected_audience.into(),
        }
    }

    pub fn verify(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(std::slice::from_ref(&self.expected_issuer));
        validation.set_audience(std::slice::from_ref(&self.expected_audience));
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        // Pin the clock-skew allowance explicitly. `jsonwebtoken` defaults to
        // 60s; leaving it implicit means a token's real lifetime is 60s longer
        // than the TTL the issuer stamped, and any test or reviewer has to know
        // that to reason about expiry.
        validation.leeway = JWT_CLOCK_SKEW_SECS;

        let data = decode::<JwtClaims>(token, &self.decoding_key, &validation)?;
        Ok(data.claims)
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    fn issuer() -> JwtIssuer {
        JwtIssuer::new(b"jwt-secret-test", "k1", "lcc.api-gateway", "lcc-api", 15)
    }

    fn verifier() -> JwtVerifier {
        JwtVerifier::new(b"jwt-secret-test", "lcc.api-gateway", "lcc-api")
    }

    #[test]
    fn round_trip() {
        let iss = issuer();
        let m = Uuid::now_v7();
        let (tok, claims) = iss.issue(&m, "owner").unwrap();
        assert_eq!(claims.sub, m.to_string());
        let v = verifier();
        let verified = v.verify(&tok).unwrap();
        assert_eq!(verified.sub, m.to_string());
        assert_eq!(verified.role, "owner");
    }

    #[test]
    fn wrong_audience() {
        let iss = JwtIssuer::new(b"k", "k", "lcc.api-gateway", "wrong", 15);
        let (tok, _) = iss.issue(&Uuid::now_v7(), "owner").unwrap();
        let v = verifier();
        assert!(v.verify(&tok).is_err());
    }
}
