//! Prometheus metrics helpers + standard service metrics.

use prometheus::{
    register_counter_vec_with_registry, register_gauge_vec_with_registry,
    register_histogram_vec_with_registry, CounterVec, GaugeVec, HistogramVec, Registry,
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("prometheus register error: {0}")]
    Register(String),
}

#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,
    /// Standard HTTP metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration_seconds: HistogramVec,
    /// Compliance Governor metrics (★)
    pub governor_evaluations_total: CounterVec,
    pub governor_evaluation_duration_seconds: HistogramVec,
    pub governor_denied_total: CounterVec,
    /// Integration Gateway metrics (★)
    pub integration_actions_total: CounterVec,
    pub integration_circuit_breaker_state: GaugeVec,
    pub integration_restriction_signals_total: CounterVec,
    /// Engagement / sequence metrics
    pub sequences_active: GaugeVec,
    pub sequences_paused_total: CounterVec,
    /// LLM call metrics
    pub llm_calls_total: CounterVec,
    pub llm_tokens_total: CounterVec,
    pub llm_cost_usd_total: CounterVec,
    pub llm_latency_seconds: HistogramVec,
}

impl Metrics {
    pub fn new(service_name: &str) -> Result<Self, MetricsError> {
        let registry = Arc::new(Registry::new());

        let http_requests_total = register_counter_vec_with_registry!(
            "lcc_http_requests_total",
            "HTTP requests by service, method, path, status",
            &["service", "method", "path", "status"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let http_request_duration_seconds = register_histogram_vec_with_registry!(
            "lcc_http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["service", "method", "path"],
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let governor_evaluations_total = register_counter_vec_with_registry!(
            "lcc_governor_evaluations_total",
            "Compliance Governor evaluations by decision",
            &["decision", "guard"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let governor_evaluation_duration_seconds = register_histogram_vec_with_registry!(
            "lcc_governor_evaluation_duration_seconds",
            "Compliance Governor evaluation duration in seconds",
            &["decision"],
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.2, 0.5, 1.0],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let governor_denied_total = register_counter_vec_with_registry!(
            "lcc_governor_denied_total",
            "Compliance Governor denials by guard",
            &["guard"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let integration_actions_total = register_counter_vec_with_registry!(
            "lcc_integration_actions_total",
            "Integration Gateway actions by track, outcome, action_type",
            &["track", "outcome", "action_type"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let integration_circuit_breaker_state = register_gauge_vec_with_registry!(
            "lcc_integration_circuit_breaker_state",
            "Circuit breaker state (0=closed, 1=half_open, 2=open)",
            &["provider", "endpoint"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let integration_restriction_signals_total = register_counter_vec_with_registry!(
            "lcc_integration_restriction_signals_total",
            "Restriction signals detected by Integration Gateway",
            &["signal_kind"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let sequences_active = register_gauge_vec_with_registry!(
            "lcc_sequences_active",
            "Currently active sequences per member",
            &["member_id"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let sequences_paused_total = register_counter_vec_with_registry!(
            "lcc_sequences_paused_total",
            "Sequences paused (e.g. by reply detection)",
            &["reason"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let llm_calls_total = register_counter_vec_with_registry!(
            "lcc_llm_calls_total",
            "LLM API calls",
            &["model_id", "tier", "outcome"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let llm_tokens_total = register_counter_vec_with_registry!(
            "lcc_llm_tokens_total",
            "LLM tokens consumed",
            &["model_id", "direction"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let llm_cost_usd_total = register_counter_vec_with_registry!(
            "lcc_llm_cost_usd_total",
            "LLM cost in USD",
            &["model_id"],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let llm_latency_seconds = register_histogram_vec_with_registry!(
            "lcc_llm_latency_seconds",
            "LLM call latency",
            &["model_id"],
            vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0],
            registry
        )
        .map_err(|e| MetricsError::Register(e.to_string()))?;

        let _ = service_name; // used in service labels at emit time
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            governor_evaluations_total,
            governor_evaluation_duration_seconds,
            governor_denied_total,
            integration_actions_total,
            integration_circuit_breaker_state,
            integration_restriction_signals_total,
            sequences_active,
            sequences_paused_total,
            llm_calls_total,
            llm_tokens_total,
            llm_cost_usd_total,
            llm_latency_seconds,
        })
    }

    /// Render metrics in Prometheus text format.
    pub fn render(&self) -> Result<String, MetricsError> {
        use prometheus::Encoder;
        let mut buf = Vec::new();
        prometheus::TextEncoder::new()
            .encode(&self.registry.gather(), &mut buf)
            .map_err(|e| MetricsError::Register(e.to_string()))?;
        String::from_utf8(buf).map_err(|e| MetricsError::Register(e.to_string()))
    }
}

pub fn counter(v: &CounterVec, labels: &[&str]) -> prometheus::Counter {
    v.with_label_values(labels)
}

pub fn gauge(v: &GaugeVec, labels: &[&str]) -> prometheus::Gauge {
    v.with_label_values(labels)
}

pub fn histogram(v: &HistogramVec, labels: &[&str]) -> prometheus::Histogram {
    v.with_label_values(labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_register() {
        let m = Metrics::new("test-svc").expect("register");
        let rendered = m.render().unwrap();
        assert!(rendered.contains("lcc_http_requests_total"));
    }

    #[test]
    fn counter_increments() {
        let m = Metrics::new("test-svc").unwrap();
        m.http_requests_total
            .with_label_values(&["test-svc", "GET", "/healthz", "200"])
            .inc();
        let rendered = m.render().unwrap();
        assert!(rendered.contains("lcc_http_requests_total"));
    }
}
