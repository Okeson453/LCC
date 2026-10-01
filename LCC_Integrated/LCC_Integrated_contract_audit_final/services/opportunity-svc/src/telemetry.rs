//! opportunity-svc tracing.

use lcc_observability::tracing_init;
use tracing::info;
pub fn init(service_name: &str, log_level: &str) {
    tracing_init::init(service_name, log_level);
    info!(service = service_name, "telemetry initialized");
}
