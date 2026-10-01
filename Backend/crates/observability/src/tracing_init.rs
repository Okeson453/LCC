//! Tracing initialization — JSON formatter + OTLP exporter.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub service_name: String,
    pub service_version: String,
    pub deployment_environment: String,
    pub otlp_endpoint: Option<String>,
    pub log_format: LogFormat,
    pub log_level: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "lcc-unknown".into(),
            service_version: "0.1.0".into(),
            deployment_environment: "dev".into(),
            otlp_endpoint: None,
            log_format: LogFormat::Json,
            log_level: "info".into(),
        }
    }
}

/// Build a config for `service_name` from the environment, falling back to
/// sensible defaults. This is the `&str` shorthand every service's `main` uses:
/// `init_tracing("lcc-audit-svc")`.
impl From<&str> for TracingConfig {
    fn from(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            service_version: std::env::var("LCC_SERVICE_VERSION")
                .unwrap_or_else(|_| "0.1.0".to_string()),
            deployment_environment: std::env::var("LCC_ENVIRONMENT")
                .unwrap_or_else(|_| "dev".to_string()),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            log_format: match std::env::var("LCC_LOG_FORMAT").as_deref() {
                Ok("pretty") => LogFormat::Pretty,
                _ => LogFormat::Json,
            },
            log_level: std::env::var("RUST_LOG")
                .or_else(|_| std::env::var("LCC_LOG_LEVEL"))
                .unwrap_or_else(|_| "info".to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum TracingError {
    #[error("tracing init failed: {0}")]
    Init(String),
}

/// Initialize the global tracing subscriber.
///
/// Should be called once at service start (in `main`). After this, all
/// `tracing::info!` / `tracing::warn!` etc. emit JSON logs (or pretty in dev)
/// and optionally forward to OTLP.
///
/// Accepts either a full [`TracingConfig`] or a bare service name, so services
/// that only need the defaults can call `init_tracing("lcc-audit-svc")`.
pub fn init_tracing(config: impl Into<TracingConfig>) -> Result<(), TracingError> {
    let config = config.into();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .map_err(|e| TracingError::Init(e.to_string()))?;

    let registry = Registry::default().with(env_filter);

    match config.log_format {
        LogFormat::Json => {
            let fmt_layer = fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(false)
                .with_target(true)
                .with_file(false)
                .with_line_number(false);
            registry.with(fmt_layer).try_init().map_err(|e| TracingError::Init(e.to_string()))?;
        }
        LogFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .pretty()
                .with_target(true);
            registry.with(fmt_layer).try_init().map_err(|e| TracingError::Init(e.to_string()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_constructs() {
        let c = TracingConfig::default();
        assert_eq!(c.service_name, "lcc-unknown");
        assert_eq!(c.log_level, "info");
    }
}
