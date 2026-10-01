//! Redis Stream consumer that fans out domain events to the per-channel
//! broadcast senders.
//!
//! Backend services publish events to the stream `lcc:realtime:events` with
//! the JSON envelope as the value. This consumer:
//!   1. Joins the consumer group `lcc-realtime`.
//!   2. Reads pending + new messages.
//!   3. Decodes each as EventEnvelope, validates against the target channel.
//!   4. Dedups via the in-process dedup cache (Redis SETNX in production).
//!   5. Sends into the per-channel broadcast sender.
//!
//! At-least-once delivery: dedup is the consumer's responsibility.

use redis::{AsyncCommands, streams::{StreamReadOptions, StreamReadReply}};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::domain::{Channel, EventEnvelope};
use crate::state::AppState;

pub const STREAM_KEY: &str = "lcc:realtime:events";
pub const GROUP: &str = "lcc-realtime";

pub async fn ensure_consumer_group(state: &AppState) -> Result<(), redis::RedisError> {
    let mut conn = state.redis().get().await.map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "pool", e.to_string())))?;
    // MKSTREAM creates the stream if it doesn't exist. Idempotent.
    let r: redis::RedisResult<String> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(STREAM_KEY)
        .arg(GROUP)
        .arg("$")
        .arg("MKSTREAM")
        .query_async(&mut conn)
        .await;
    match r {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("BUSYGROUP") => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn run_consumer(state: AppState) {
    info!("realtime consumer starting");
    if let Err(e) = ensure_consumer_group(&state).await {
        error!(error = %e, "ensure_consumer_group failed; consumer will retry");
    }

    loop {
        if let Err(e) = poll_once(&state).await {
            error!(error = %e, "realtime poll failed");
            sleep(Duration::from_millis(500)).await;
        } else {
            // No events → small sleep to avoid busy-loop.
            sleep(Duration::from_millis(50)).await;
        }
    }
}

async fn poll_once(state: &AppState) -> Result<(), redis::RedisError> {
    let mut conn = state.redis().get().await.map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "pool", e.to_string())))?;

    let opts = StreamReadOptions::default()
        .group(GROUP, &state.config().subscriber_id)
        .count(100)
        .block(50);

    let reply: StreamReadReply = conn
        .xread_options(&[STREAM_KEY], &[">"], &opts)
        .await?;

    for stream_key in reply.keys {
        for entry in stream_key.ids {
            let stream_id = entry.id.clone();
            // Each entry has one field named "envelope" containing the JSON.
            let envelope_json = match entry.map.get("envelope") {
                Some(v) => match v {
                    redis::Value::BulkString(b) => match std::str::from_utf8(b) {
                        Ok(s) => s.to_string(),
                        Err(e) => {
                            warn!(error = %e, "envelope is not UTF-8; skipping");
                            ack(&mut conn, &stream_id).await?;
                            continue;
                        }
                    },
                    redis::Value::SimpleString(s) => s.clone(),
                    _ => {
                        warn!("envelope field has unexpected type; skipping");
                        ack(&mut conn, &stream_id).await?;
                        continue;
                    }
                },
                None => {
                    warn!(id = %stream_id, "entry missing 'envelope' field");
                    ack(&mut conn, &stream_id).await?;
                    continue;
                }
            };

            let envelope: EventEnvelope = match serde_json::from_str(&envelope_json) {
                Ok(e) => e,
                Err(e) => {
                    warn!(error = %e, id = %stream_id, "envelope JSON decode failed");
                    ack(&mut conn, &stream_id).await?;
                    continue;
                }
            };

            // Determine the target channel from the producer service. The
            // producer service is part of the envelope; we route by event_name.
            // The realtime-svc keeps a small mapping table from event_name to
            // target channel. Unknown events are acked (dropped) to avoid
            // blocking the stream.
            let target = match route_event(&envelope.event_name) {
                Some(c) => c,
                None => {
                    ack(&mut conn, &stream_id).await?;
                    continue;
                }
            };

            if let Err(reason) = envelope.validate_for(target) {
                warn!(error = %reason, id = %stream_id, "envelope failed validation");
                ack(&mut conn, &stream_id).await?;
                continue;
            }

            // Dedup.
            {
                let mut dedup = state.dedup().await;
                let now = chrono::Utc::now();
                // Opportunistically prune expired entries.
                let ttl = chrono::Duration::seconds(state.config().event_dedup_ttl_secs as i64);
                dedup.retain(|_, t| now.signed_duration_since(*t) < ttl);
                if dedup.insert(envelope.event_id, now).is_some() {
                    // Already seen.
                    ack(&mut conn, &stream_id).await?;
                    continue;
                }
            }

            // Fan out.
            let sender = state.broadcast(target);
            // `send` returns Err only if there are no receivers. That's
            // fine — just skip.
            let _ = sender.send(envelope);

            ack(&mut conn, &stream_id).await?;
        }
    }
    Ok(())
}

async fn ack(
    conn: &mut deadpool_redis::Connection,
    stream_id: &str,
) -> Result<(), redis::RedisError> {
    let _: i64 = redis::cmd("XACK")
        .arg(STREAM_KEY)
        .arg(GROUP)
        .arg(stream_id)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Map event names to the channel that owns them.
pub fn route_event(event_name: &str) -> Option<Channel> {
    match event_name {
        "briefing.refresh" | "briefing.section.updated" => Some(Channel::Briefing),
        "approval.created" | "approval.expired" | "approval.bulk_decided" => Some(Channel::Approvals),
        "engagement.inbound.received"
        | "engagement.task.created"
        | "engagement.draft_ready"
        | "engagement.task.expired" => Some(Channel::Engagement),
        "compliance.restriction_detected"
        | "compliance.restriction_cleared"
        | "compliance.config_activated"
        | "compliance.circuit_breaker_state_changed" => Some(Channel::Compliance),
        "sequence.reply_detected"
        | "sequence.paused"
        | "sequence.resumed"
        | "sequence.completed"
        | "sequence.step.sent"
        | "sequence.step.failed" => Some(Channel::Sequence),
        _ => None,
    }
}

/// Test helper: publish a synthetic event (skips Redis for unit tests).
#[allow(dead_code)]
pub async fn publish_test_event(state: &AppState, env: EventEnvelope) {
    let ch = match route_event(&env.event_name) {
        Some(c) => c,
        None => return,
    };
    if env.validate_for(ch).is_err() {
        return;
    }
    let _ = state.broadcast(ch).send(env);
}

/// Build a publish helper for upstream services (compile-time module that
/// constructs the envelope and writes it to the stream).
pub fn envelope(
    event_name: &str,
    member_id: uuid::Uuid,
    producer: &str,
    payload: serde_json::Value,
) -> EventEnvelope {
    EventEnvelope {
        event_id: uuid::Uuid::new_v4(),
        event_name: event_name.to_string(),
        occurred_at: chrono::Utc::now(),
        member_id,
        producer_service: producer.to_string(),
        trace_id: uuid::Uuid::new_v4().to_string(),
        payload,
    }
}

/// Map a free-form name → canonical channel name (used by the SSE/WS
/// subscribe frames).
pub fn channel_from_id_str(s: &str) -> Option<Channel> {
    let trimmed = s.trim_start_matches('/').trim_start_matches("api/v1/ws/");
    Channel::from_id(trimmed).or_else(|| Channel::from_id(s))
}

/// Build a HashMap of default producer→channel mappings (for diagnostics).
#[allow(dead_code)]
pub fn default_route_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("briefing.refresh", "ws.briefing");
    m.insert("briefing.section.updated", "ws.briefing");
    m.insert("approval.created", "ws.approvals");
    m.insert("approval.expired", "ws.approvals");
    m.insert("approval.bulk_decided", "ws.approvals");
    m.insert("engagement.inbound.received", "ws.engagement");
    m.insert("engagement.task.created", "ws.engagement");
    m.insert("engagement.draft_ready", "ws.engagement");
    m.insert("engagement.task.expired", "ws.engagement");
    m.insert("compliance.restriction_detected", "ws.compliance");
    m.insert("compliance.restriction_cleared", "ws.compliance");
    m.insert("compliance.config_activated", "ws.compliance");
    m.insert("compliance.circuit_breaker_state_changed", "ws.compliance");
    m.insert("sequence.reply_detected", "ws.sequence");
    m.insert("sequence.paused", "ws.sequence");
    m.insert("sequence.resumed", "ws.sequence");
    m.insert("sequence.completed", "ws.sequence");
    m.insert("sequence.step.sent", "ws.sequence");
    m.insert("sequence.step.failed", "ws.sequence");
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_table_covers_all_contract_events() {
        let expected = [
            "briefing.refresh", "briefing.section.updated",
            "approval.created", "approval.expired", "approval.bulk_decided",
            "engagement.inbound.received", "engagement.task.created",
            "engagement.draft_ready", "engagement.task.expired",
            "compliance.restriction_detected", "compliance.restriction_cleared",
            "compliance.config_activated", "compliance.circuit_breaker_state_changed",
            "sequence.reply_detected", "sequence.paused", "sequence.resumed",
            "sequence.completed", "sequence.step.sent", "sequence.step.failed",
        ];
        for e in expected {
            assert!(route_event(e).is_some(), "missing route for {e}");
        }
    }

    #[test]
    fn unknown_event_routes_to_none() {
        assert!(route_event("totally.unrelated").is_none());
        assert!(route_event("").is_none());
    }

    #[test]
    fn envelope_validation_rejects_wrong_channel() {
        let env = envelope(
            "approval.created",
            uuid::Uuid::new_v4(),
            "approval-svc",
            serde_json::json!({}),
        );
        assert!(env.validate_for(Channel::Briefing).is_err());
        assert!(env.validate_for(Channel::Approvals).is_ok());
    }
}
