//! Permit-token — JWT-like signed claims with Ed25519 (asymmetric).
//!
//! ## Algorithm choice — Ed25519, not HS256
//!
//! HS256 is a symmetric MAC: both issuer (Compliance Governor) and verifier
//! (Integration Gateway) must hold the *same secret*. That means any compromise
//! of the gateway exposes the governor's signing capability. It's the wrong
//! primitive for this architecture.
//!
//! Ed25519 is asymmetric: the governor holds a private key used only to
//! sign, and the gateway holds the matching public key used only to verify.
//! The gateway cannot forge permits even if compromised. The governor can
//! rotate the key without coordinating with the gateway (the gateway only
//! needs the new public key, distributed via the audit-svc on rotation).
//!
//! ## Token shape
//!
//! Header:
//! ```
//! { "alg": "EdDSA", "typ": "JWT", "kid": "<key-id>" }
//! ```
//!
//! Claims:
//! ```
//! {
//!   "iss": "compliance-governor",
//!   "aud": "integration-gateway",
//!   "sub": "<member_id>",
//!   "act": "<action_id>",
//!   "typ": "<action_type>",     // e.g. "post_publish"
//!   "risk": "<low|medium|high>",
//!   "appr": "<approval_id|null>",
//!   "cv":   "<config_version>",
//!   "iat":  <unix>,
//!   "exp":  <unix>,
//!   "jti":  "<uuid>"
//! }
//! ```
//!
//! TTL is bounded to ≤60 seconds (`ttl_seconds` in the builder).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PermitError {
    #[error("malformed token: {0}")]
    Malformed(String),
    #[error("invalid signature")]
    BadSignature,
    #[error("token expired")]
    Expired,
    #[error("wrong audience: expected {expected}, got {actual}")]
    WrongAudience { expected: String, actual: String },
    #[error("wrong issuer: expected {expected}, got {actual}")]
    WrongIssuer { expected: String, actual: String },
    #[error("unknown key id: {0}")]
    UnknownKey(String),
    #[error("key not configured")]
    NoKey,
}

// ---- Claims ----

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermitClaims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub act: String,
    pub typ: String,
    pub risk: String,
    pub appr: Option<String>,
    pub cv: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
}

impl PermitClaims {
    pub fn member_id(&self) -> &str { &self.sub }
    pub fn action_id(&self) -> &str { &self.act }
    pub fn action_type(&self) -> &str { &self.typ }
    pub fn risk_tier(&self) -> &str { &self.risk }
    pub fn config_version(&self) -> &str { &self.cv }
}

pub struct PermitClaimsBuilder {
    iss: String,
    aud: String,
    sub: String,
    act: String,
    typ: String,
    risk: String,
    appr: Option<String>,
    cv: String,
    ttl_seconds: i64,
    now: i64,
}

impl PermitClaimsBuilder {
    pub fn new(
        member_id: impl Into<String>,
        action_id: impl Into<String>,
        action_type: impl Into<String>,
        risk_tier: impl Into<String>,
        config_version: impl Into<String>,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            iss: "compliance-governor".into(),
            aud: "integration-gateway".into(),
            sub: member_id.into(),
            act: action_id.into(),
            typ: action_type.into(),
            risk: risk_tier.into(),
            appr: None,
            cv: config_version.into(),
            ttl_seconds: 60,
            now,
        }
    }

    pub fn approval_id(mut self, id: Option<String>) -> Self { self.appr = id; self }
    pub fn ttl_seconds(mut self, ttl: i64) -> Self { self.ttl_seconds = ttl; self }
    pub fn audience(mut self, aud: impl Into<String>) -> Self { self.aud = aud.into(); self }
    pub fn issuer(mut self, iss: impl Into<String>) -> Self { self.iss = iss.into(); self }

    pub fn build(self) -> PermitClaims {
        PermitClaims {
            iss: self.iss,
            aud: self.aud,
            sub: self.sub,
            act: self.act,
            typ: self.typ,
            risk: self.risk,
            appr: self.appr,
            cv: self.cv,
            iat: self.now,
            exp: self.now + self.ttl_seconds,
            jti: Uuid::new_v4().to_string(),
        }
    }
}

// ---- Key registry ----

pub trait KeyStore: Send + Sync {
    /// Returns the private key for the active key id (signer side only).
    fn sign_with(&self, kid: &str) -> Result<SigningKey, PermitError>;
    /// Returns the public key for a given key id (verifier side).
    fn verify_with(&self, kid: &str) -> Result<VerifyingKey, PermitError>;
    /// Returns the active (signing) key id.
    fn active_kid(&self) -> String;
}

#[derive(Clone)]
pub struct StaticKeyStore {
    keys: std::collections::HashMap<String, (SigningKey, VerifyingKey)>,
    active: String,
}

impl StaticKeyStore {
    pub fn from_signing_key(key: SigningKey) -> Self {
        let verifying = key.verifying_key();
        let mut keys = std::collections::HashMap::new();
        let kid = "k1".to_string();
        keys.insert(kid.clone(), (key, verifying));
        Self { keys, active: kid }
    }

    pub fn add_key(&mut self, kid: impl Into<String>, key: SigningKey) {
        let verifying = key.verifying_key();
        self.keys.insert(kid.into(), (key, verifying));
    }

    pub fn set_active(&mut self, kid: impl Into<String>) {
        self.active = kid.into();
    }
}

impl KeyStore for StaticKeyStore {
    fn sign_with(&self, kid: &str) -> Result<SigningKey, PermitError> {
        self.keys
            .get(kid)
            .map(|(s, _)| s.clone())
            .ok_or_else(|| PermitError::UnknownKey(kid.to_string()))
    }

    fn verify_with(&self, kid: &str) -> Result<VerifyingKey, PermitError> {
        self.keys
            .get(kid)
            .map(|(_, v)| *v)
            .ok_or_else(|| PermitError::UnknownKey(kid.to_string()))
    }

    fn active_kid(&self) -> String {
        self.active.clone()
    }
}

// ---- Signer (Compliance Governor side) ----

#[derive(Clone)]
pub struct PermitSigner {
    keys: Arc<dyn KeyStore>,
    audience: String,
}

impl PermitSigner {
    pub fn new(keys: Arc<dyn KeyStore>, audience: impl Into<String>) -> Self {
        Self { keys, audience: audience.into() }
    }

    pub fn issue(&self, claims: &PermitClaims) -> Result<String, PermitError> {
        // Sanity: claims must target the configured audience.
        if claims.aud != self.audience {
            return Err(PermitError::WrongAudience {
                expected: self.audience.clone(),
                actual: claims.aud.clone(),
            });
        }
        let kid = self.keys.active_kid();
        let signing_key = self.keys.sign_with(&kid)?;

        let header = serde_json::json!({
            "alg": "EdDSA",
            "typ": "JWT",
            "kid": kid,
        });
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap_or_default());
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap_or_default());
        let signing_input = format!("{header_b64}.{claims_b64}");

        let signature = signing_key.sign(signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{signing_input}.{sig_b64}"))
    }
}

// ---- Verifier (Integration Gateway side) ----

#[derive(Clone)]
pub struct PermitVerifier {
    keys: Arc<dyn KeyStore>,
    expected_audience: String,
}

impl PermitVerifier {
    pub fn new(keys: Arc<dyn KeyStore>, expected_audience: impl Into<String>) -> Self {
        Self { keys, expected_audience: expected_audience.into() }
    }

    pub fn verify(&self, token: &str) -> Result<PermitClaims, PermitError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(PermitError::Malformed(format!("expected 3 parts, got {}", parts.len())));
        }
        let header_b64 = parts[0];
        let claims_b64 = parts[1];
        let sig_b64 = parts[2];

        let header: serde_json::Value = URL_SAFE_NO_PAD
            .decode(header_b64)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .ok_or_else(|| PermitError::Malformed("bad header".into()))?;

        let alg = header.get("alg").and_then(|v| v.as_str()).unwrap_or("");
        if alg != "EdDSA" {
            return Err(PermitError::Malformed(format!("unsupported alg: {alg}")));
        }
        let kid = header.get("kid").and_then(|v| v.as_str()).unwrap_or("");
        let verifying_key = self.keys.verify_with(kid)?;

        let claims: PermitClaims = URL_SAFE_NO_PAD
            .decode(claims_b64)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .ok_or_else(|| PermitError::Malformed("bad claims".into()))?;

        if claims.aud != self.expected_audience {
            return Err(PermitError::WrongAudience {
                expected: self.expected_audience.clone(),
                actual: claims.aud,
            });
        }
        if claims.iss != "compliance-governor" {
            return Err(PermitError::WrongIssuer {
                expected: "compliance-governor".into(),
                actual: claims.iss,
            });
        }

        let signing_input = format!("{header_b64}.{claims_b64}");
        let sig_bytes = URL_SAFE_NO_PAD
            .decode(sig_b64)
            .map_err(|_| PermitError::Malformed("bad signature encoding".into()))?;
        let sig_arr: [u8; 64] = sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| PermitError::Malformed("sig must be 64 bytes".into()))?;
        let signature = Signature::from_bytes(&sig_arr);

        verifying_key
            .verify(signing_input.as_bytes(), &signature)
            .map_err(|_| PermitError::BadSignature)?;

        let now = Utc::now().timestamp();
        if now >= claims.exp {
            return Err(PermitError::Expired);
        }

        Ok(claims)
    }
}

// ---- Convenience constructors ----

/// Generate a fresh Ed25519 keypair. Used at startup to provision the key
/// from a Vault secret.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    use rand::rngs::OsRng;
    let mut csprng = OsRng;
    let signing = SigningKey::generate(&mut csprng);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

/// Public-key-only verifier. Used by the Integration Gateway, which never
/// has the signing key.
pub fn public_only_verifier(public_key: VerifyingKey, audience: impl Into<String>) -> PermitVerifier {
    struct OnlyOne(VerifyingKey);
    impl KeyStore for OnlyOne {
        fn sign_with(&self, _: &str) -> Result<SigningKey, PermitError> { Err(PermitError::NoKey) }
        fn verify_with(&self, kid: &str) -> Result<VerifyingKey, PermitError> {
            if kid == "k1" { Ok(self.0) } else { Err(PermitError::UnknownKey(kid.into())) }
        }
        fn active_kid(&self) -> String { "k1".into() }
    }
    let keys: Arc<dyn KeyStore> = Arc::new(OnlyOne(public_key));
    PermitVerifier::new(keys, audience)
}

/// Signer with a real keypair. Used by the Compliance Governor.
pub fn signing_only_signer(signing_key: SigningKey, audience: impl Into<String>) -> PermitSigner {
    let store = StaticKeyStore::from_signing_key(signing_key);
    let keys: Arc<dyn KeyStore> = Arc::new(store);
    PermitSigner::new(keys, audience)
}
