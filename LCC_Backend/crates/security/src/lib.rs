//! `lcc-security` — Vault client, AES-GCM envelope encryption, mTLS helpers.

pub mod envelope;
pub mod mtls;
pub mod vault;

pub use envelope::{decrypt_token, encrypt_token, EncryptedToken, Nonce};
pub use mtls::{load_mtls_cert, MtlsCert};
pub use vault::{SecretRef, VaultClient, VaultError};
