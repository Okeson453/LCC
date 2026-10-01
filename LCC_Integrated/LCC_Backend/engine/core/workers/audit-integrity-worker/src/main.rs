//! audit-integrity-worker main.

use lcc_audit_integrity_worker::{run, Config};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env();
    info!(service = %cfg.service_name, "starting");
    run(cfg).await
}
