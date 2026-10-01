//! mTLS cert loading and rotation helpers.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MtlsError {
    #[error("io error reading cert: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid PEM: {0}")]
    InvalidPem(String),
    #[error("cert-manager error: {0}")]
    CertManager(String),
}

/// MtlsCert — a cert + key pair (or PEM bundle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsCert {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_path: Option<PathBuf>,
}

impl MtlsCert {
    /// Load cert from disk. Verifies the files exist and are non-empty.
    pub fn load(cert_path: impl AsRef<Path>, key_path: impl AsRef<Path>) -> Result<Self, MtlsError> {
        let cert_path = cert_path.as_ref().to_path_buf();
        let key_path = key_path.as_ref().to_path_buf();

        let cert = std::fs::read(&cert_path)?;
        if cert.is_empty() {
            return Err(MtlsError::InvalidPem("empty cert file".into()));
        }
        if !cert.starts_with(b"-----BEGIN") {
            return Err(MtlsError::InvalidPem("not a PEM".into()));
        }

        let key = std::fs::read(&key_path)?;
        if key.is_empty() {
            return Err(MtlsError::InvalidPem("empty key file".into()));
        }
        if !key.starts_with(b"-----BEGIN") {
            return Err(MtlsError::InvalidPem("key not a PEM".into()));
        }

        Ok(Self {
            cert_path,
            key_path,
            ca_path: None,
        })
    }

    pub fn with_ca(mut self, ca_path: impl AsRef<Path>) -> Self {
        self.ca_path = Some(ca_path.as_ref().to_path_buf());
        self
    }

    /// In production, this returns a tonic::transport::ServerTlsConfig.
    /// In dev, callers can ignore the return and use plaintext (mTLS optional).
    pub fn as_pem_strings(&self) -> Result<(String, String), MtlsError> {
        let cert = std::fs::read_to_string(&self.cert_path)?;
        let key = std::fs::read_to_string(&self.key_path)?;
        Ok((cert, key))
    }
}

pub fn load_mtls_cert(
    cert_path: impl AsRef<Path>,
    key_path: impl AsRef<Path>,
) -> Result<MtlsCert, MtlsError> {
    MtlsCert::load(cert_path, key_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_pem_rejected() {
        let dir = tempdir();
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");
        std::fs::write(&cert, b"not a pem").unwrap();
        std::fs::write(&key, b"not a pem either").unwrap();
        let err = MtlsCert::load(&cert, &key).unwrap_err();
        assert!(matches!(err, MtlsError::InvalidPem(_)));
    }

    fn tempdir() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "lcc-mtls-test-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
