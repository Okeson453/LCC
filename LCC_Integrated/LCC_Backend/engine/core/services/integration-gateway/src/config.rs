//! Service-specific config for the Integration Gateway.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationGatewayConfig {
    pub http_port: u16,
    pub grpc_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub vault_addr: String,
    pub vault_token: String,
    /// Base64-encoded 32-byte Ed25519 **public** key for verifying permit-tokens.
    /// The Integration Gateway NEVER holds the corresponding private key
    /// (asymmetric split — see ADR-0003).
    pub permit_pubkey_b64: String,
    pub compliance_config_path: String,
    pub track_b_ws_addr: String,
}

impl IntegrationGatewayConfig {
    pub fn from_env() -> Self {
        Self {
            http_port: std::env::var("HTTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            grpc_port: std::env::var("GRPC_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50051),
            database_url: std::env::var("DATABASE_URL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "postgres://postgres:postgres@postgres:5432/lcc".into()),
            redis_url: std::env::var("REDIS_URL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "redis://redis:6379".into()),
            vault_addr: std::env::var("VAULT_ADDR")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "https://vault:8200".into()),
            // No default token — must be supplied via ExternalSecret (F-14/F-15).
            vault_token: std::env::var("VAULT_TOKEN").unwrap_or_default(),
            permit_pubkey_b64: std::env::var("PERMIT_VERIFY_PUBKEY_B64").unwrap_or_default(),
            compliance_config_path: std::env::var("COMPLIANCE_CONFIG_PATH")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "/etc/lcc/compliance/ccfg-active.yaml".into()),
            track_b_ws_addr: std::env::var("TRACK_B_WS_ADDR")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "0.0.0.0:8443".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_parse() {
        let c = IntegrationGatewayConfig::from_env();
        assert_eq!(c.http_port, 8080);
    }

    #[test]
    fn empty_env_vars_fall_back() {
        std::env::set_var("HTTP_PORT", "");
        std::env::set_var("DATABASE_URL", "");
        std::env::set_var("REDIS_URL", "");
        std::env::set_var("VAULT_ADDR", "");
        let c = IntegrationGatewayConfig::from_env();
        assert_eq!(c.http_port, 8080);
        assert!(!c.database_url.is_empty());
        std::env::remove_var("HTTP_PORT");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("REDIS_URL");
        std::env::remove_var("VAULT_ADDR");
    }
}
