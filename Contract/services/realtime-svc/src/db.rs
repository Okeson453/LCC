//! Realtime service state (DB pool + Redis pool).

use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub config: Config,
    pub db_pool: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    /// Per-member connection registry (member_id -> active connection count).
    /// Enforces `per_member_conn_limit`.
    pub member_conns: tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, usize>>,
    /// Per-channel broadcast fan-out: a single `tokio::sync::broadcast` per
    /// channel; subscribers create a fresh `Receiver` per connection.
    pub broadcasts:
        std::collections::HashMap<crate::domain::Channel, tokio::sync::broadcast::Sender<crate::domain::EventEnvelope>>,
    /// In-process dedup cache for events (event_id -> seen timestamp).
    /// In production this lives in Redis; the in-process cache is the
    /// hot path for tests.
    pub dedup: tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, chrono::DateTime<chrono::Utc>>>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&config.database_url)
            .await?;

        let redis = deadpool_redis::Config::from_url(&config.redis_url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| sqlx::Error::Configuration(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("redis: {e}"),
            ))))?;

        let mut broadcasts = std::collections::HashMap::new();
        for ch in [
            crate::domain::Channel::Briefing,
            crate::domain::Channel::Approvals,
            crate::domain::Channel::Engagement,
            crate::domain::Channel::Compliance,
            crate::domain::Channel::Sequence,
        ] {
            let (tx, _rx) = tokio::sync::broadcast::channel::<crate::domain::EventEnvelope>(1024);
            broadcasts.insert(ch, tx);
        }

        Ok(Self(Arc::new(Inner {
            config,
            db_pool,
            redis,
            member_conns: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            broadcasts,
            dedup: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        })))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }
    pub fn db(&self) -> &sqlx::PgPool {
        &self.0.db_pool
    }
    pub fn redis(&self) -> &deadpool_redis::Pool {
        &self.0.redis
    }
    pub fn broadcast(&self, ch: crate::domain::Channel) -> &tokio::sync::broadcast::Sender<crate::domain::EventEnvelope> {
        self.0.broadcasts.get(&ch).expect("channel sender")
    }
    pub async fn member_conns(&self) -> tokio::sync::MutexGuard<'_, std::collections::HashMap<uuid::Uuid, usize>> {
        self.0.member_conns.lock().await
    }
    pub async fn dedup(&self) -> tokio::sync::MutexGuard<'_, std::collections::HashMap<uuid::Uuid, chrono::DateTime<chrono::Utc>>> {
        self.0.dedup.lock().await
    }
}
