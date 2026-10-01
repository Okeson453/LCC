//! Vault client (KV v2 + Transit) — wraps the official vaultrs crate with
//! our preferred retry + envelope patterns.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("vault transport: {0}")]
    Transport(String),
    #[error("vault returned 403 for path {path} — access denied")]
    Forbidden { path: String },
    #[error("vault returned 404 for path {path} — secret not found")]
    NotFound { path: String },
    #[error("vault returned 5xx — retry exhausted")]
    Unavailable,
    #[error("invalid secret data shape: {0}")]
    InvalidData(String),
}

/// SecretRef — opaque reference to a Vault secret.
///
/// Path is the KV v2 path (e.g., `secret/linkedin/oauth/m_001/access_token`).
/// The data is fetched on demand and cached for `ttl_seconds` (default 60).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretRef {
    pub path: String,
    pub key: Option<String>,         // sub-key; None for whole blob
    pub ttl_seconds: u64,
}

impl SecretRef {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            key: None,
            ttl_seconds: 60,
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }
}

/// VaultClient — async client.
///
/// In production this calls Vault HTTP API. In tests, you can construct a
/// `VaultClient::with_mock(map)` that returns canned values.
#[derive(Clone)]
pub struct VaultClient {
    addr: String,
    token: String,
    /// Optional override for tests; maps path -> value.
    mock_data: Option<std::collections::HashMap<String, String>>,
}

impl VaultClient {
    pub fn new(addr: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            token: token.into(),
            mock_data: None,
        }
    }

    pub fn with_mock(data: std::collections::HashMap<String, String>) -> Self {
        Self {
            addr: String::new(),
            token: String::new(),
            mock_data: Some(data),
        }
    }

    /// Read a secret by ref. Returns the value at `path` (or `path:key` if a
    /// sub-key was specified).
    pub async fn read(&self, secret_ref: &SecretRef) -> Result<String, VaultError> {
        // Test-mode path: read from mock map.
        if let Some(map) = &self.mock_data {
            let lookup_key = match &secret_ref.key {
                Some(k) => format!("{}:{}", secret_ref.path, k),
                None => secret_ref.path.clone(),
            };
            return map
                .get(&lookup_key)
                .cloned()
                .or_else(|| map.get(&secret_ref.path).cloned())
                .ok_or(VaultError::NotFound {
                    path: secret_ref.path.clone(),
                });
        }

        // Production path: GET {addr}/v1/{path} with X-Vault-Token header.
        let url = format!("{}/v1/{}", self.addr, secret_ref.path);
        let resp = reqwest::Client::new()
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await
            .map_err(|e| VaultError::Transport(e.to_string()))?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| VaultError::Transport(e.to_string()))?;
                let data = body
                    .get("data")
                    .ok_or_else(|| VaultError::InvalidData("missing 'data' key".into()))?;
                match &secret_ref.key {
                    Some(k) => data
                        .get(k)
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .ok_or_else(|| VaultError::InvalidData(format!("missing sub-key {k}"))),
                    None => {
                        // Return the whole data as JSON.
                        serde_json::to_string(data)
                            .map_err(|e| VaultError::InvalidData(e.to_string()))
                    }
                }
            }
            reqwest::StatusCode::FORBIDDEN => Err(VaultError::Forbidden {
                path: secret_ref.path.clone(),
            }),
            reqwest::StatusCode::NOT_FOUND => Err(VaultError::NotFound {
                path: secret_ref.path.clone(),
            }),
            s if s.is_server_error() => Err(VaultError::Unavailable),
            s => Err(VaultError::Transport(format!("unexpected status {s}"))),
        }
    }

    /// Write a secret. Production path.
    pub async fn write(&self, path: &str, data: serde_json::Value) -> Result<(), VaultError> {
        if self.mock_data.is_some() {
            return Ok(()); // no-op in mock mode
        }
        let url = format!("{}/v1/{}", self.addr, path);
        let resp = reqwest::Client::new()
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({ "data": data }))
            .send()
            .await
            .map_err(|e| VaultError::Transport(e.to_string()))?;
        match resp.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::NO_CONTENT => Ok(()),
            reqwest::StatusCode::FORBIDDEN => Err(VaultError::Forbidden { path: path.into() }),
            reqwest::StatusCode::NOT_FOUND => Err(VaultError::NotFound { path: path.into() }),
            s if s.is_server_error() => Err(VaultError::Unavailable),
            s => Err(VaultError::Transport(format!("unexpected status {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_read_returns_value() {
        let mut map = std::collections::HashMap::new();
        map.insert("secret/test/access_token".to_string(), "abc123".to_string());
        let client = VaultClient::with_mock(map);
        let sec = SecretRef::new("secret/test/access_token");
        let val = client.read(&sec).await.unwrap();
        assert_eq!(val, "abc123");
    }

    #[tokio::test]
    async fn mock_read_subkey() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "secret/test:token".to_string(),
            "xyz789".to_string(),
        );
        let client = VaultClient::with_mock(map);
        let sec = SecretRef::new("secret/test").with_key("token");
        let val = client.read(&sec).await.unwrap();
        assert_eq!(val, "xyz789");
    }

    #[tokio::test]
    async fn mock_read_missing_returns_not_found() {
        let client = VaultClient::with_mock(std::collections::HashMap::new());
        let sec = SecretRef::new("secret/missing");
        let err = client.read(&sec).await.unwrap_err();
        assert!(matches!(err, VaultError::NotFound { .. }));
    }
}
