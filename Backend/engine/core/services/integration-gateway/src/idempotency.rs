//! Idempotency store — Redis + Postgres-backed replay protection.
//!
//! Every write-side Integration Layer call carries an `idempotency_key` of the
//! form `<action_type>:<resource_id>:<version>`. The store is checked first;
//! if the key has been seen, the cached response is returned without
//! re-executing.

use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A stored idempotency record: the cached response to replay for a key.
type IdempotencyRow = (
    String,            // key
    i32,               // response_status
    serde_json::Value, // response_body
    DateTime<Utc>,     // created_at
    DateTime<Utc>,     // expires_at
);

#[derive(Debug, Error)]
pub enum IdempotencyError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    /// Pool checkout failures arrive wrapped by deadpool, so they are not
    /// convertible into the bare `redis::RedisError` variant above.
    #[error("redis pool: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),
    #[error("postgres: {0}")]
    Postgres(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotentResult {
    pub key: String,
    pub response_status: u16,
    pub response_body: serde_json::Value,
    pub stored_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct IdempotencyStore {
    redis: deadpool_redis::Pool,
    postgres: sqlx::PgPool,
}

impl IdempotencyStore {
    pub fn new(redis: deadpool_redis::Pool, postgres: sqlx::PgPool) -> Self {
        Self { redis, postgres }
    }

    /// Returns Some(cached) if the key has been seen; None if first time.
    pub async fn get(&self, key: &str) -> Result<Option<IdempotentResult>, IdempotencyError> {
        // Check Redis first (fast path).
        let mut conn = self.redis.get().await?;
        let cached: Option<String> = conn.get(format!("idem:{key}")).await?;
        if let Some(json) = cached {
            if let Ok(result) = serde_json::from_str::<IdempotentResult>(&json) {
                return Ok(Some(result));
            }
        }

        // Fall back to Postgres (durable mirror).
        let row: Option<IdempotencyRow> = sqlx::query_as(
            r#"
                SELECT key, response_status, response_body, created_at, expires_at
                FROM idempotency_key
                WHERE key = $1 AND expires_at > now()
                "#,
        )
        .bind(key)
        .fetch_optional(&self.postgres)
        .await?;
        if let Some((k, status, body, stored_at, expires_at)) = row {
            Ok(Some(IdempotentResult {
                key: k,
                response_status: status as u16,
                response_body: body,
                stored_at,
                expires_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Store the result for a key with a 7-day TTL.
    pub async fn put(
        &self,
        key: &str,
        response_status: u16,
        response_body: &serde_json::Value,
    ) -> Result<(), IdempotencyError> {
        let now = Utc::now();
        let expires_at = now + Duration::days(7);

        let result = IdempotentResult {
            key: key.to_string(),
            response_status,
            response_body: response_body.clone(),
            stored_at: now,
            expires_at,
        };
        let json = serde_json::to_string(&result).unwrap_or_default();

        // Redis fast path.
        let mut conn = self.redis.get().await?;
        let _: () = conn
            .set_ex(format!("idem:{key}"), json, 7 * 24 * 3600)
            .await?;

        // Postgres durable mirror.
        let request_hash = hash_key(key);
        sqlx::query(
            r#"
            INSERT INTO idempotency_key (
                key, method, path, request_hash, response_status, response_body,
                created_at, expires_at
            )
            VALUES ($1, 'POST', '/integrations/execute', $2, $3, $4::jsonb, $5, $6)
            ON CONFLICT (key) DO UPDATE
              SET response_status = EXCLUDED.response_status,
                  response_body = EXCLUDED.response_body
            "#,
        )
        .bind(key)
        .bind(request_hash)
        .bind(response_status as i32)
        .bind(response_body)
        .bind(now)
        .bind(expires_at)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }
}

fn hash_key(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let a = hash_key("abc");
        let b = hash_key("abc");
        assert_eq!(a, b);
        let c = hash_key("def");
        assert_ne!(a, c);
    }
}
