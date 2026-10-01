//! identity-svc configuration.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub http_port: u16,
    pub linkedin_client_id: String,
    pub linkedin_client_secret: String,
    pub public_oauth_redirect_uri: String,
    pub auth_jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub access_token_ttl_secs: u32,
    pub refresh_token_ttl_secs: u32,
    pub token_encryption_key: String,
    #[serde(skip)]
    pub http_client: reqwest::Client,
    /// In-memory set of unused refresh-jti values; in production this
    /// lives in Redis. Used by Service::mark_jti_unused.
    #[serde(skip)]
    pub unused_jti: std::sync::Arc<Mutex<std::collections::HashSet<String>>>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("http client: {e}"))?;

        let linkedin_client_id = std::env::var("LINKEDIN_CLIENT_ID").unwrap_or_default();
        let linkedin_client_secret = std::env::var("LINKEDIN_CLIENT_SECRET").unwrap_or_default();
        let public_oauth_redirect_uri = std::env::var("LINKEDIN_REDIRECT_URI")
            .unwrap_or_else(|_| "https://api.lcc.okeson.example/api/v1/auth/linkedin/callback".into());

        let auth_jwt_secret = std::env::var("LCC_AUTH_JWT_SECRET").unwrap_or_default();
        if auth_jwt_secret.len() < 32 {
            return Err(
                "LCC_AUTH_JWT_SECRET must be set and at least 32 characters in non-local environments"
                    .into(),
            );
        }

        let token_encryption_key = std::env::var("LCC_TOKEN_ENCRYPTION_KEY").unwrap_or_default();
        if token_encryption_key.len() < 16 {
            return Err("LCC_TOKEN_ENCRYPTION_KEY must be at least 16 chars".into());
        }

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://lcc_app:dev_lcc_app@postgres:5432/lcc".into()),
            http_port: parse_port("LCC_HTTP_PORT").unwrap_or(8090),
            linkedin_client_id,
            linkedin_client_secret,
            public_oauth_redirect_uri,
            auth_jwt_secret,
            jwt_issuer: std::env::var("LCC_JWT_ISSUER").unwrap_or_else(|_| "lcc-auth".into()),
            jwt_audience: std::env::var("LCC_JWT_AUDIENCE").unwrap_or_else(|_| "lcc-api".into()),
            access_token_ttl_secs: parse_u32("LCC_ACCESS_TOKEN_TTL_SECS").unwrap_or(900),    // 15 min
            refresh_token_ttl_secs: parse_u32("LCC_REFRESH_TOKEN_TTL_SECS").unwrap_or(2_592_000), // 30 d
            token_encryption_key,
            http_client,
            unused_jti: std::sync::Arc::new(Mutex::new(std::collections::HashSet::new())),
        })
    }

    /// Build a minimal in-memory config for local development without
    /// external services. Refuses to start if LCC_ENVIRONMENT != local.
    pub fn dev_local() -> Result<Self, String> {
        std::env::set_var("LCC_ENVIRONMENT", "local");
        std::env::set_var(
            "LCC_AUTH_JWT_SECRET",
            "dev-secret-only-for-local-do-not-use-in-prod",
        );
        std::env::set_var("LCC_TOKEN_ENCRYPTION_KEY", "0123456789abcdef-local");
        std::env::set_var("LINKEDIN_CLIENT_ID", "local-stub-client");
        std::env::set_var("LINKEDIN_CLIENT_SECRET", "local-stub-secret");
        Self::from_env()
    }
}

fn parse_port(name: &str) -> Option<u16> {
    std::env::var(name).ok().and_then(|s| s.parse().ok())
}
fn parse_u32(name: &str) -> Option<u32> {
    std::env::var(name).ok().and_then(|s| s.parse().ok())
}
