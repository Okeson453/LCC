//! data-purge-worker — TTL-based data purger.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

// F-AUDIT-36: this module was never declared, so `src/logic.rs` was not
// compiled into the crate and `run()`'s call to `purge_expired_data` was a
// compile error. The worker could not be built at all.
pub mod logic;

use logic::purge_expired_data;

use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service_name: String,
    pub database_url: String,
    pub redis_url: String,
    pub poll_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: "data-purge-worker".into(),
            database_url: "postgresql://postgres:postgres@postgres:5432/lcc".into(),
            redis_url: "redis://redis:6379".into(),
            poll_interval_seconds: 60,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self::default()
    }
}

pub async fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    info!(service = %cfg.service_name, interval = cfg.poll_interval_seconds, "started");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&cfg.database_url)
        .await?;

    loop {
        match purge_expired_data(&pool).await {
            Ok(n) => info!(processed = n, "tick complete"),
            Err(e) => error!(error = %e, "tick failed"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(cfg.poll_interval_seconds)).await;
    }
}
