//! Consumer — async event-bus consumer with idempotency dedup.

use async_trait::async_trait;
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use std::sync::Arc;
use thiserror::Error;

use super::envelope::Envelope;
use super::topics::Topic;

#[derive(Debug, Error)]
pub enum ConsumerError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("pool error: {0}")]
    Pool(String),
    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("handler error: {0}")]
    Handler(String),
}

/// ConsumerHandler — implement on a domain service to process envelopes.
#[async_trait]
pub trait ConsumerHandler: Send + Sync {
    async fn handle(&self, envelope: Envelope) -> Result<(), ConsumerError>;
}

/// Consumer — subscribes to a topic and dispatches to a handler.
pub struct Consumer {
    pool: RedisPool,
    consumer_group: String,
    consumer_name: String,
    stream_prefix: String,
}

impl Consumer {
    pub fn new(
        pool: RedisPool,
        consumer_group: impl Into<String>,
        consumer_name: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            consumer_group: consumer_group.into(),
            consumer_name: consumer_name.into(),
            stream_prefix: "lcc".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.stream_prefix = prefix.into();
        self
    }

    /// Ensure the consumer group exists (XGROUP CREATE ... MKSTREAM).
    pub async fn ensure_group(&self, topic: Topic) -> Result<(), ConsumerError> {
        let stream_key = format!("{}:{}", self.stream_prefix, topic);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ConsumerError::Pool(e.to_string()))?;
        // Ignore BUSYGROUP error (group already exists).
        let res: redis::RedisResult<String> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&stream_key)
            .arg(&self.consumer_group)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;
        if let Err(e) = res {
            if !e.to_string().contains("BUSYGROUP") {
                return Err(ConsumerError::Redis(e));
            }
        }
        Ok(())
    }

    /// Run a single polling cycle: read up to `batch_size` events, dispatch
    /// to the handler, ack via XACK on success.
    pub async fn poll_once<H: ConsumerHandler>(
        &self,
        topic: Topic,
        handler: Arc<H>,
        batch_size: usize,
    ) -> Result<u32, ConsumerError> {
        let stream_key = format!("{}:{}", self.stream_prefix, topic);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ConsumerError::Pool(e.to_string()))?;

        let entries: Vec<(String, Vec<(String, String)>)> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.consumer_group)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg(batch_size)
            .arg("BLOCK")
            .arg(0u64)
            .arg("STREAMS")
            .arg(&stream_key)
            .arg(">")
            .query_async(&mut conn)
            .await?;

        let mut processed = 0u32;
        for (stream_id, fields) in entries {
            // Reconstruct Envelope from fields.
            let mut event_id = None;
            let mut data = None;
            let mut topic_str = None;
            let mut trace_id = None;
            let mut producer = None;
            let mut member_id = None;
            let mut idempotency_key = None;
            for (k, v) in fields {
                match k.as_str() {
                    "event_id" => event_id = Some(v),
                    "data" => data = Some(v),
                    "topic" => topic_str = Some(v),
                    "trace_id" => trace_id = Some(v),
                    "producer" => producer = Some(v),
                    "member_id" => member_id = Some(v),
                    "idempotency_key" => idempotency_key = Some(v),
                    _ => {}
                }
            }

            let data = match data {
                Some(d) => d,
                None => {
                    // Ack and skip — malformed entry.
                    let _: i64 = conn.xack(&stream_key, &self.consumer_group, &[&stream_id]).await?;
                    continue;
                }
            };

            // Idempotency check via Redis SET NX.
            if let Some(ref key) = idempotency_key {
                let dedup_key = format!("dedup:{}:{}", self.consumer_group, key);
                let set: redis::RedisResult<bool> = redis::cmd("SET")
                    .arg(&dedup_key)
                    .arg("1")
                    .arg("NX")
                    .arg("EX")
                    .arg(7 * 24 * 3600)
                    .query_async(&mut conn)
                    .await;
                if matches!(set, Ok(false)) {
                    // Already processed — ack and skip.
                    let _: i64 = conn.xack(&stream_key, &self.consumer_group, &[&stream_id]).await?;
                    continue;
                }
            }

            // Parse the envelope.
            let envelope: Envelope = match serde_json::from_str(&data) {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!(error = %err, stream_id = %stream_id, "failed to parse envelope; acking");
                    let _: i64 = conn.xack(&stream_key, &self.consumer_group, &[&stream_id]).await?;
                    continue;
                }
            };

            match handler.handle(envelope).await {
                Ok(()) => {
                    let _: i64 = conn.xack(&stream_key, &self.consumer_group, &[&stream_id]).await?;
                    processed += 1;
                }
                Err(e) => {
                    // Don't ack — message will be re-delivered. Caller decides on retry budget.
                    tracing::error!(
                        error = %e,
                        stream_id = %stream_id,
                        topic = ?topic,
                        "handler failed; message will be re-delivered"
                    );
                }
            }

            // Touch consumer-side fields used above to silence unused warnings
            // when the envelope was parsed from `data` and these header fields
            // are not used (they live in the inner Envelope struct).
            let _ = (event_id, topic_str, trace_id, producer, member_id);
        }

        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pool built from a URL connects lazily, so this needs no live Redis.
    fn lazy_pool() -> RedisPool {
        deadpool_redis::Config::from_url("redis://127.0.0.1:6379")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("pool config")
    }

    #[test]
    fn consumer_group_constants() {
        let consumer = Consumer::new(lazy_pool(), "test-group", "test-consumer");
        assert_eq!(consumer.consumer_group, "test-group");
        assert_eq!(consumer.consumer_name, "test-consumer");
        assert_eq!(consumer.stream_prefix, "lcc");
    }

    #[test]
    fn stream_prefix_is_overridable() {
        let consumer = Consumer::new(lazy_pool(), "g", "c").with_prefix("custom");
        assert_eq!(consumer.stream_prefix, "custom");
    }
}
