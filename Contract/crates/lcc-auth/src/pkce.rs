//! PKCE — Proof Key for Code Exchange (RFC 7636) helpers.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// PKCE pair: code_verifier + code_challenge.
#[derive(Debug, Clone)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// Generate a fresh PKCE pair using the S256 method (RFC 7636 §4.2).
///
/// The verifier is 32 bytes of cryptographic randomness, base64url-no-pad
/// encoded (43 chars). The challenge is SHA-256 of the verifier, base64url-no-pad.
pub fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Derive the S256 challenge from a verifier.
pub fn derive_code_challenge(verifier: &str) -> String {
    let mut h = Sha256::new();
    h.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(h.finalize())
}

/// Convenience: generate both at once.
pub fn generate_pkce_pair() -> PkcePair {
    let verifier = generate_code_verifier();
    let challenge = derive_code_challenge(&verifier);
    PkcePair {
        code_verifier: verifier,
        code_challenge: challenge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_length() {
        let v = generate_code_verifier();
        assert_eq!(v.len(), 43); // 32 bytes → 43 base64url-no-pad chars
    }

    #[test]
    fn challenge_is_sha256_of_verifier() {
        let v = "test-verifier-string";
        let expected = {
            let mut h = Sha256::new();
            h.update(v.as_bytes());
            URL_SAFE_NO_PAD.encode(h.finalize())
        };
        assert_eq!(derive_code_challenge(v), expected);
    }

    #[test]
    fn pair_is_consistent() {
        let pair = generate_pkce_pair();
        assert_eq!(
            derive_code_challenge(&pair.code_verifier),
            pair.code_challenge
        );
    }

    #[test]
    fn pair_verifiers_are_unique() {
        let p1 = generate_pkce_pair();
        let p2 = generate_pkce_pair();
        assert_ne!(p1.code_verifier, p2.code_verifier);
        assert_ne!(p1.code_challenge, p2.code_challenge);
    }
}
