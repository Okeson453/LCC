//! Typed environment-variable loader (envy + serde).

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Local,
    Dev,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str_or_default(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            "dev" | "development" => Self::Dev,
            _ => Self::Local,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub service_name: String,
    pub service_version: String,
    pub environment: Environment,
    pub log_level: String,
    pub http_port: u16,
    pub grpc_port: u16,

    pub database_url: String,
    pub database_max_connections: u32,

    pub redis_url: String,

    pub governor_endpoint: String,
    pub integration_endpoint: String,
    pub identity_endpoint: String,

    pub ai_worker_endpoint: String,
    pub opportunity_intel_endpoint: String,
    pub kb_intel_endpoint: String,
    pub voice_intel_endpoint: String,
    pub scoring_intel_endpoint: String,

    pub compliance_config_path: String,
    pub jwt_secret: String,
    pub permit_token_secret: String,
    pub vault_addr: String,
    pub vault_token: String,
    pub otlp_endpoint: Option<String>,
    pub enable_mtls: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u16_or(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_u32_or(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_bool_or(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|s| matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

/// Load typed EnvConfig from the process environment. Fails if any required
/// field cannot be parsed.
pub fn load_env_config() -> Result<EnvConfig, envy::Error> {
    let env_obj = envy::from_env::<RawEnv>()
        .map_err(|e| envy::Error::Custom(format!("env parse: {e}")))?;
    Ok(env_obj.into())
}

#[derive(Debug, Deserialize)]
struct RawEnv {
    #[serde(default = "default_service_name")]
    service_name: String,
    #[serde(default = "default_service_version")]
    service_version: String,
    #[serde(default = "default_environment")]
    environment: String,
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_http_port")]
    http_port: u16,
    #[serde(default = "default_grpc_port")]
    grpc_port: u16,
    #[serde(default)]
    database_url: String,
    #[serde(default = "default_db_max_conn")]
    database_max_connections: u32,
    #[serde(default)]
    redis_url: String,
    #[serde(default)]
    governor_endpoint: String,
    #[serde(default)]
    integration_endpoint: String,
    #[serde(default)]
    identity_endpoint: String,
    #[serde(default)]
    ai_worker_endpoint: String,
    #[serde(default)]
    opportunity_intel_endpoint: String,
    #[serde(default)]
    kb_intel_endpoint: String,
    #[serde(default)]
    voice_intel_endpoint: String,
    #[serde(default)]
    scoring_intel_endpoint: String,
    #[serde(default)]
    compliance_config_path: String,
    #[serde(default)]
    jwt_secret: String,
    #[serde(default)]
    permit_token_secret: String,
    #[serde(default)]
    vault_addr: String,
    #[serde(default)]
    vault_token: String,
    #[serde(default)]
    otlp_endpoint: Option<String>,
    #[serde(default)]
    enable_mtls: bool,
}

fn default_service_name() -> String {
    env_or("SERVICE_NAME", "lcc-unknown")
}
fn default_service_version() -> String {
    env_or("SERVICE_VERSION", "0.1.0")
}
fn default_environment() -> String {
    env_or("LCC_ENV", "local")
}
fn default_log_level() -> String {
    env_or("RUST_LOG", "info")
}
fn default_http_port() -> u16 {
    env_u16_or("HTTP_PORT", 8080)
}
fn default_grpc_port() -> u16 {
    env_u16_or("GRPC_PORT", 50051)
}
fn default_db_max_conn() -> u32 {
    env_u32_or("DATABASE_MAX_CONNECTIONS", 10)
}

impl From<RawEnv> for EnvConfig {
    fn from(r: RawEnv) -> Self {
        Self {
            service_name: r.service_name,
            service_version: r.service_version,
            environment: Environment::from_str_or_default(&r.environment),
            log_level: r.log_level,
            http_port: r.http_port,
            grpc_port: r.grpc_port,
            database_url: r.database_url,
            database_max_connections: r.database_max_connections,
            redis_url: r.redis_url,
            governor_endpoint: r.governor_endpoint,
            integration_endpoint: r.integration_endpoint,
            identity_endpoint: r.identity_endpoint,
            ai_worker_endpoint: r.ai_worker_endpoint,
            opportunity_intel_endpoint: r.opportunity_intel_endpoint,
            kb_intel_endpoint: r.kb_intel_endpoint,
            voice_intel_endpoint: r.voice_intel_endpoint,
            scoring_intel_endpoint: r.scoring_intel_endpoint,
            compliance_config_path: r.compliance_config_path,
            jwt_secret: r.jwt_secret,
            permit_token_secret: r.permit_token_secret,
            vault_addr: r.vault_addr,
            vault_token: r.vault_token,
            otlp_endpoint: r.otlp_endpoint,
            enable_mtls: r.enable_mtls,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_parsing() {
        assert_eq!(Environment::from_str_or_default("PRODUCTION"), Environment::Production);
        assert_eq!(Environment::from_str_or_default("staging"), Environment::Staging);
        assert_eq!(Environment::from_str_or_default("dev"), Environment::Dev);
        assert_eq!(Environment::from_str_or_default("anything-else"), Environment::Local);
    }
}
