//! Postgres connection pool builder + configuration.

use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub log_statements: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/lcc".to_string(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_seconds: 5,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
            log_statements: false,
        }
    }
}

/// Build a sqlx PgPool with the given configuration.
///
/// Connect-options are tuned for the LCC workload (RLS context per request,
/// short statement logging, application_name for pg_stat_activity tracing).
pub async fn build_pool(config: &PoolConfig) -> Result<PgPool, sqlx::Error> {
    let connect_opts = PgConnectOptions::from_str(&config.url)?
        .application_name("lcc-core")
        .log_statements(if config.log_statements {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Off
        });

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
        .idle_timeout(Some(Duration::from_secs(config.idle_timeout_seconds)))
        .max_lifetime(Some(Duration::from_secs(config.max_lifetime_seconds)))
        .connect_with(connect_opts)
        .await?;

    // Verify connection with a SELECT 1.
    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}
