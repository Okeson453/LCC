//! staleness-scanner — tier-aware stale-data scanner.
//!
//! Audit fixes:
//! - F-AUDIT-14: `logic` was never declared as a module, yet `run()` called
//!   `scan_stale_data` from it, so the crate could not compile.
//! - F-AUDIT-15: `Config::from_env()` returned `Self::default()` and read
//!   nothing from the environment, so `database_url` was always the hardcoded
//!   localhost-compose default and the poll interval was never tunable. Every
//!   worker and service in this workspace has the same defect.

#![deny(warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

pub mod logic;

use serde::Deserialize;
use std::time::Duration;
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
            service_name: "staleness-scanner".into(),
            database_url: "postgresql://postgres:postgres@postgres:5432/lcc".into(),
            redis_url: "redis://redis:6379".into(),
            poll_interval_seconds: 60,
        }
    }
}

impl Config {
    /// Read configuration from the environment, falling back to the local
    /// compose defaults.
    ///
    /// F-AUDIT-15: this previously returned `Self::default()` unconditionally,
    /// so no deployment could override the database URL or the poll interval.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("LCC_SERVICE_NAME") {
            if !v.is_empty() {
                cfg.service_name = v;
            }
        }
        if let Ok(v) = std::env::var("LCC_DATABASE_URL") {
            if !v.is_empty() {
                cfg.database_url = v;
            }
        }
        if let Ok(v) = std::env::var("LCC_REDIS_URL") {
            if !v.is_empty() {
                cfg.redis_url = v;
            }
        }
        if let Ok(v) = std::env::var("LCC_POLL_INTERVAL_SECONDS") {
            if let Ok(n) = v.parse::<u64>() {
                if n > 0 {
                    cfg.poll_interval_seconds = n;
                }
            }
        }
        cfg
    }
}

pub async fn run(cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    info!(service = %cfg.service_name, interval = cfg.poll_interval_seconds, "started");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&cfg.database_url)
        .await?;

    let interval = Duration::from_secs(cfg.poll_interval_seconds);
    loop {
        match logic::run_pass(&pool).await {
            Ok(n) => info!(marked = n, "tick complete"),
            Err(e) => error!(error = %e, "tick failed"),
        }
        tokio::time::sleep(interval).await;
    }
}
