//! `lcc-observability` — tracing init, Prometheus metrics, log redaction.

pub mod metrics;
pub mod redaction;
pub mod span;
pub mod tracing_init;

pub use metrics::{counter, gauge, histogram, Metrics, MetricsError};
pub use redaction::{redact_log_fields, RedactedField};
pub use span::{current_trace_id, span_for_governor_evaluate, span_for_integration_execute};
pub use tracing_init::{init_tracing, TracingConfig};
