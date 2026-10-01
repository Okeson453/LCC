//! Publisher — async publisher to Redis Streams (Phase 1-2) or Kafka (Phase 3+).

use deadpool_redis::Pool as RedisPool;
use serde::Serialize;
use thiserror::Error;

use super::envelope::Envelope;
use super::topics::Topic;

#[derive(Debug, Error)]
pub enum PublisherError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("pool error: {0}")]
    Pool(String),
    #[error("serialization: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Publisher — wraps a Redis Pool. Each service constructs one at start and
/// shares via AppState.
#[derive(Clone)]
pub struct Publisher {
    pool: RedisPool,
    /// Stream key prefix — defaults to "lcc".
    stream_prefix: String,
}

impl Publisher {
    pub fn new(pool: RedisPool) -> Self {
        Self {
            pool,
            stream_prefix: "lcc".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.stream_prefix = prefix.into();
        self
    }

    /// Publish an envelope to its topic's stream.
    pub async fn publish(&self, envelope: &Envelope) -> Result<String, PublisherError> {
        let stream_key = format!("{}:{}", self.stream_prefix, envelope.header.topic);
        let payload = serde_json::to_string(envelope)?;
        let idem = envelope.header.idempotency_key.clone();

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| PublisherError::Pool(e.to_string()))?;

        // XADD <stream> * event_id <uuid> topic <topic> data <json> idempotency_key <key>
        let stream_id: String = redis::cmd("XADD")
            .arg(&stream_key)
            .arg("*")
            .arg("event_id")
            .arg(envelope.header.event_id.to_string())
            .arg("topic")
            .arg(envelope.header.topic.as_str())
            .arg("trace_id")
            .arg(&envelope.header.trace_id)
            .arg("producer")
            .arg(&envelope.header.producer_service)
            .arg("member_id")
            .arg(envelope.header.member_id.clone().unwrap_or_default())
            .arg("schema_version")
            .arg(envelope.header.schema_version.to_string())
            .arg("idempotency_key")
            .arg(&idem)
            .arg("data")
            .arg(&payload)
            .query_async(&mut conn)
            .await?;

        Ok(stream_id)
    }

    /// Publish a typed payload — convenience wrapper.
    pub async fn publish_typed<T: Serialize>(
        &self,
        topic: Topic,
        producer: &str,
        trace_id: &str,
        payload: &T,
    ) -> Result<String, PublisherError> {
        // Construct a generic envelope (using JSON value as the data field).
        let json = serde_json::to_string(payload)?;
        let envelope_json = serde_json::json!({
            "data": json,
            "topic": topic.as_str(),
        });
        let stream_key = format!("{}:{}", self.stream_prefix, topic);
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| PublisherError::Pool(e.to_string()))?;
        let stream_id: String = redis::cmd("XADD")
            .arg(&stream_key)
            .arg("*")
            .arg("topic")
            .arg(topic.as_str())
            .arg("producer")
            .arg(producer)
            .arg("trace_id")
            .arg(trace_id)
            .arg("data")
            .arg(envelope_json.to_string())
            .query_async(&mut conn)
            .await?;
        Ok(stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_key_format() {
        // Smoke test that we don't accidentally crash on topic names.
        for topic in [
            Topic::MemberCreated,
            Topic::ContentItemApproved,
            Topic::ComplianceConfigActivated,
        ] {
            let key = format!("lcc:{}", topic);
            assert!(key.starts_with("lcc:"));
        }
    }
}
