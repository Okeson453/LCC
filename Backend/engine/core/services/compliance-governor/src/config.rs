//! Service-specific config wiring for Compliance Governor.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorServiceConfig {
    /// Logical service name, used for tracing, metrics and the
    /// `x-service` header. Fixed per service, not per deployment.
    pub service_name: String,
    /// Where this service sends audit events.
    pub audit_svc_url: String,
    /// The Compliance Governor, which every external action is gated on.
    pub compliance_governor_url: String,

    pub http_port: u16,
    pub grpc_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub compliance_config_path: String,
    /// Dev/test only: 32-byte seed used to deterministically derive the
    /// Ed25519 signing key. Production deployments use PERMIT_SIGNING_KEY_B64.
    pub permit_signing_seed: Option<String>,
    pub scoring_intel_endpoint: String,
}
/// Documented local-development defaults. The ports match the `containerPort`
/// pinned in `infra/k8s/base/*.yaml`, and the upstream URLs match the
/// api-gateway's `Default`, so a local stack and a k8s stack agree.
impl Default for GovernorServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "compliance-governor".into(),
            audit_svc_url: "http://audit-svc:8091".into(),
            compliance_governor_url: "http://compliance-governor:8080".into(),
            http_port: 8080,
            grpc_port: 8080,
            database_url: "postgres://lcc:lcc@postgres:5432/lcc".into(),
            redis_url: "redis://redis:6379".into(),
            compliance_config_path: "config/ccfg-local.yaml".into(),
            permit_signing_seed: None,
            scoring_intel_endpoint: "http://scoring-intel:8090".into(),
        }
    }
}

impl GovernorServiceConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: "compliance-governor".into(),
            audit_svc_url: std::env::var("LCC_AUDIT_SVC_URL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://audit-svc:8091".into()),
            compliance_governor_url: std::env::var("LCC_COMPLIANCE_GOVERNOR_URL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://compliance-governor:8080".into()),
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
            compliance_config_path: std::env::var("COMPLIANCE_CONFIG_PATH")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "/etc/lcc/compliance/ccfg-active.yaml".into()),
            permit_signing_seed: std::env::var("PERMIT_SIGNING_SEED")
                .ok()
                .filter(|s| !s.is_empty()),
            scoring_intel_endpoint: std::env::var("SCORING_INTEL_ENDPOINT")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://scoring-intel:8093".into()),
        }
    }

    /// Load the active compliance config from the configured path.
    pub fn load_compliance_config(
        &self,
    ) -> Result<
        lcc_compliance::config::ComplianceConfig,
        lcc_compliance::config::ComplianceConfigError,
    > {
        lcc_compliance::config::ComplianceConfig::from_yaml_file(Path::new(
            &self.compliance_config_path,
        ))
    }
}

/// Alias so tooling and tests can refer to every service's config by the
/// same name.
pub type Config = GovernorServiceConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_defaults() {
        let c = GovernorServiceConfig::from_env();
        assert_eq!(c.http_port, 8080);
        assert_eq!(c.grpc_port, 50051);
    }

    #[test]
    fn empty_env_vars_are_skipped() {
        // Setting an env var to "" should fall back to defaults, not use the empty string.
        std::env::set_var("HTTP_PORT", "");
        std::env::set_var("DATABASE_URL", "");
        std::env::set_var("REDIS_URL", "");
        std::env::set_var("COMPLIANCE_CONFIG_PATH", "");
        std::env::set_var("SCORING_INTEL_ENDPOINT", "");
        let c = GovernorServiceConfig::from_env();
        assert_eq!(c.http_port, 8080);
        assert!(!c.database_url.is_empty());
        assert!(!c.redis_url.is_empty());
        assert!(!c.compliance_config_path.is_empty());
        std::env::remove_var("HTTP_PORT");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("REDIS_URL");
        std::env::remove_var("COMPLIANCE_CONFIG_PATH");
        std::env::remove_var("SCORING_INTEL_ENDPOINT");
    }
}
