//! sequence-step-scheduler main.

use lcc_sequence_step_scheduler::{run, Config};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cfg = Config::from_env();
    info!(service = %cfg.service_name, "starting");
    run(cfg).await
}
