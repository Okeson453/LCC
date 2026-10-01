//! Realtime service configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub auth_jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    /// Channel names that this instance serves.
    pub channels: Vec<String>,
    /// Subscriber ID used to claim the Redis Stream consumer group. Unique
    /// per pod so events are load-balanced across replicas.
    pub subscriber_id: String,
    /// Maximum concurrent connections per member.
    pub per_member_conn_limit: usize,
    /// Heartbeat interval (seconds) — server pings clients on this cadence.
    pub heartbeat_seconds: u64,
    /// Idle timeout (seconds) — server closes the connection after this many
    /// seconds of no traffic.
    pub idle_timeout_seconds: u64,
    /// Retention for dedup state (events older than this are dropped on
    /// dedup-collision check).
    pub event_dedup_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let channels: Vec<String> = [
            "ws.briefing",
            "ws.approvals",
            "ws.engagement",
            "ws.compliance",
            "ws.sequence",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let auth_jwt_secret = std::env::var("LCC_AUTH_JWT_SECRET").unwrap_or_default();
        if auth_jwt_secret.is_empty() {
            return Err(
                "LCC_AUTH_JWT_SECRET must be set (shared with api-gateway and identity-svc)".into(),
            );
        }

        Ok(Self {
            http_port: parse_port("LCC_HTTP_PORT").unwrap_or(8093),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://lcc_app:dev_lcc_app@postgres:5432/lcc".into()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".into()),
            auth_jwt_secret,
            jwt_issuer: std::env::var("LCC_JWT_ISSUER").unwrap_or_else(|_| "lcc-auth".into()),
            jwt_audience: std::env::var("LCC_JWT_AUDIENCE").unwrap_or_else(|_| "lcc-api".into()),
            channels,
            subscriber_id: std::env::var("HOSTNAME").unwrap_or_else(|_| {
                format!("realtime-{}", uuid::Uuid::new_v4())
            }),
            per_member_conn_limit: 5,
            heartbeat_seconds: 30,
            idle_timeout_seconds: 120,
            event_dedup_ttl_secs: 7 * 24 * 3600,
        })
    }

    pub fn channel_set(&self) -> HashSet<String> {
        self.channels.iter().cloned().collect()
    }
}

fn parse_port(name: &str) -> Option<u16> {
    std::env::var(name).ok().and_then(|s| s.parse().ok())
}
