//! Connection pool for upstream HTTP services.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub base_url: String,
    pub timeout_ms: u64,
    pub max_connections: usize,
}

impl UpstreamConfig {
    /// Build an upstream config with the gateway's standard tuning.
    ///
    /// `base_url` is normalised (trailing `/` stripped) so path joining in
    /// `forward` never produces a double slash.
    pub fn new(name: &str, base_url: &str) -> Self {
        Self::with_tuning(
            name,
            base_url,
            crate::state::UPSTREAM_TIMEOUT_MS,
            crate::state::UPSTREAM_MAX_CONNECTIONS,
        )
    }

    /// Build an upstream config with explicit tuning, for callers that need
    /// to deviate from the gateway-wide defaults.
    pub fn with_tuning(
        name: &str,
        base_url: &str,
        timeout_ms: u64,
        max_connections: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout_ms,
            max_connections,
        }
    }
}

#[derive(Debug)]
pub struct UpstreamClient {
    pub config: UpstreamConfig,
    pub http: reqwest::Client,
}

impl UpstreamClient {
    pub fn new(config: UpstreamConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(config.max_connections)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, http }
    }

    /// Forward a request to this upstream.
    ///
    /// Retained for direct callers; the gateway's HTTP handlers use
    /// `crate::proxy::forward`, which additionally handles header
    /// allow-listing and response translation.
    pub async fn forward(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<reqwest::Body>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http.request(method, url);
        if let Some(b) = body {
            req = req.body(b);
        }
        req.send().await
    }
}

pub type SharedPool = Arc<UpstreamClient>;

impl UpstreamClient {
    /// The upstream service name (e.g., "identity-svc", "content-svc").
    pub fn upstream_name(&self) -> &str {
        &self.config.name
    }
}

pub fn build_pools(configs: Vec<UpstreamConfig>) -> Vec<SharedPool> {
    configs
        .into_iter()
        .map(|c| Arc::new(UpstreamClient::new(c)))
        .collect()
}
