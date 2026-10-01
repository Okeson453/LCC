//! Tracing/telemetry init.

use lcc_observability::tracing_init::{self, LogFormat, TracingConfig, TracingError};
use tracing::info;

pub fn init(service_name: &str, log_level: &str) {
    let config = TracingConfig {
        service_name: service_name.to_string(),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
        deployment_environment: std::env::var("LCC_ENVIRONMENT")
            .unwrap_or_else(|_| "local".to_string()),
        otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        log_format: LogFormat::Json,
        log_level: log_level.to_string(),
    };

    if let Err(TracingError::Init(msg)) = tracing_init::init_tracing(config) {
        // Tracing is best-effort: a bad RUST_LOG must not stop the gateway
        // from serving, so this is reported and startup continues without the
        // global subscriber.
        eprintln!("[{service_name}] tracing init failed: {msg}");
    }
    info!(service = service_name, "telemetry initialized");
}
