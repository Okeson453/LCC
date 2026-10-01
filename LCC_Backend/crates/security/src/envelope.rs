//! AES-256-GCM envelope encryption for tokens-at-rest (Source Backend Design
//! Concept §44, §45).
//!
//! Tokens (LinkedIn OAuth, LLM provider keys, etc.) are encrypted at rest in
//! the database. The DEK (data encryption key) is wrapped by a KEK held in
//! Vault/KMS. For dev/test, a static 256-bit key may be used; production must
//! use a KMS-wrapped DEK.
//!
//! ## F-87 — AAD is mandatory
//!
//! AES-GCM authenticated encryption binds the ciphertext to **Additional
//! Authenticated Data (AAD)**. Encrypting without AAD means an attacker who
//! can move ciphertext rows around can re-use them in places they don't
//! belong (e.g., swap a `linkedin_oauth_token` ciphertext into the
//! `anthropic_api_key` slot). Every `encrypt_token` call MUST pass an AAD
//! identifying the field purpose; `decrypt_token` MUST verify the same AAD.
//! On AAD mismatch the call fails with `EnvelopeError::Decrypt` (loud, never
//! silent).

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvelopeError {
    #[error("encryption failed: {0}")]
    Encrypt(String),
    #[error("decryption failed: {0}")]
    Decrypt(String),
    #[error("invalid key length: expected 32 bytes, got {0}")]
    InvalidKey(usize),
    #[error("invalid ciphertext length: {0}")]
    InvalidCiphertext(usize),
    #[error("missing AAD")]
    MissingAad,
    #[error("aad mismatch — ciphertext belongs to a different field")]
    AadMismatch,
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
}

/// EncryptedToken — wire format: base64(nonce || ciphertext).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedToken(String);

impl EncryptedToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Pre-defined AAD labels. Each label identifies a specific field the token
/// is allowed to decrypt into. A label is the canonical string passed as AAD.
///
/// Production may derive these from a registry in `lcc-internal/config/`.
pub mod aad {
    pub const LINKEDIN_OAUTH_TOKEN: &[u8] = b"lcc:encrypted:linkedin_oauth_token:v1";
    pub const LINKEDIN_REFRESH_TOKEN: &[u8] = b"lcc:encrypted:linkedin_refresh_token:v1";
    pub const ANTHROPIC_API_KEY: &[u8] = b"lcc:encrypted:anthropic_api_key:v1";
    pub const OPENAI_API_KEY: &[u8] = b"lcc:encrypted:openai_api_key:v1";
    pub const PERMIT_SIGNING_KEY: &[u8] = b"lcc:encrypted:permit_signing_key:v1";
    pub const PERMIT_VERIFY_PUBKEY: &[u8] = b"lcc:encrypted:permit_verify_pubkey:v1";
    pub const GENERAL_SECRET: &[u8] = b"lcc:encrypted:general_secret:v1";
}

/// Encrypt a plaintext token with the given 256-bit key. The nonce is fresh
/// per encryption (random 96 bits).
///
/// F-87: `aad` is mandatory. Pass a label from the [`aad`] module, or any
/// purpose-specific bytes that the caller can re-supply on decrypt.
pub fn encrypt_token(
    plaintext: &str,
    key_bytes: &[u8],
    aad: &[u8],
) -> Result<EncryptedToken, EnvelopeError> {
    if aad.is_empty() {
        return Err(EnvelopeError::MissingAad);
    }
    if key_bytes.len() != 32 {
        return Err(EnvelopeError::InvalidKey(key_bytes.len()));
    }
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad,
            },
        )
        .map_err(|e| EnvelopeError::Encrypt(e.to_string()))?;

    // nonce (12) || ciphertext
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    Ok(EncryptedToken(STANDARD.encode(&out)))
}

/// Decrypt a token. The nonce is the first 12 bytes.
///
/// F-87: AAD must match exactly; otherwise the call fails with
/// `AadMismatch`. There is no fallback — we will not silently re-decrypt
/// against an empty label.
pub fn decrypt_token(
    encrypted: &EncryptedToken,
    key_bytes: &[u8],
    aad: &[u8],
) -> Result<String, EnvelopeError> {
    if aad.is_empty() {
        return Err(EnvelopeError::MissingAad);
    }
    if key_bytes.len() != 32 {
        return Err(EnvelopeError::InvalidKey(key_bytes.len()));
    }
    let raw = STANDARD.decode(encrypted.as_str())?;
    if raw.len() < 12 {
        return Err(EnvelopeError::InvalidCiphertext(raw.len()));
    }

    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&raw[..12]);
    let ciphertext = &raw[12..];

    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| EnvelopeError::AadMismatch)?;

    String::from_utf8(plaintext).map_err(|e| EnvelopeError::Decrypt(e.to_string()))
}

/// Backwards-compatible helper that requires AAD callers to opt in. New code
/// MUST use [`encrypt_token`] directly.
#[deprecated(note = "F-87: pass AAD. Use encrypt_token(plaintext, key, aad).")]
pub fn encrypt_token_legacy(plaintext: &str, key_bytes: &[u8]) -> Result<EncryptedToken, EnvelopeError> {
    encrypt_token(plaintext, key_bytes, aad::GENERAL_SECRET)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Vec<u8> {
        (0..32u8).collect()
    }

    #[test]
    fn round_trip_with_aad() {
        let k = key();
        let enc = encrypt_token("my-secret-token", &k, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        let dec = decrypt_token(&enc, &k, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        assert_eq!(dec, "my-secret-token");
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let k1 = key();
        let k2: Vec<u8> = (32..64u8).collect();
        let enc = encrypt_token("my-secret", &k1, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        assert!(decrypt_token(&enc, &k2, aad::LINKEDIN_OAUTH_TOKEN).is_err());
    }

    #[test]
    fn wrong_aad_fails_to_decrypt_f87() {
        let k = key();
        let enc = encrypt_token("my-secret", &k, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        // Trying to decrypt as if it belonged to a different field.
        let err = decrypt_token(&enc, &k, aad::ANTHROPIC_API_KEY).expect_err("must reject");
        assert!(matches!(err, EnvelopeError::AadMismatch | EnvelopeError::Decrypt(_)));
    }

    #[test]
    fn empty_aad_rejected() {
        let k = key();
        let err = encrypt_token("x", &k, b"").expect_err("must reject");
        assert!(matches!(err, EnvelopeError::MissingAad));
        let err2 = decrypt_token(
            &EncryptedToken("AAAA".into()),
            &k,
            b"",
        )
        .expect_err("must reject");
        assert!(matches!(err2, EnvelopeError::MissingAad | EnvelopeError::Base64(_)));
    }

    #[test]
    fn nonce_is_random_per_call() {
        let k = key();
        let e1 = encrypt_token("same plaintext", &k, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        let e2 = encrypt_token("same plaintext", &k, aad::LINKEDIN_OAUTH_TOKEN).unwrap();
        assert_ne!(e1.0, e2.0, "nonces must be fresh");
    }

    #[test]
    fn invalid_key_length() {
        assert!(matches!(
            encrypt_token("x", &[0u8; 16], aad::LINKEDIN_OAUTH_TOKEN),
            Err(EnvelopeError::InvalidKey(16))
        ));
    }
}
