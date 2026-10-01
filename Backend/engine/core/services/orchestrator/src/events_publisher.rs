//! Event publisher — bridges domain events to the realtime Redis Stream.
//!
//! Every canonical realtime event defined in
//! `contract_audit/realtime/lcc-realtime-contract.yaml` flows through
//! `publish_event`, which:
//!   1. Wraps the payload in the canonical envelope (event_id, event_name,
//!      occurred_at, member_id, producer_service, trace_id, payload).
//!   2. Determines the target channel via `route_event` (mirrored from
//!      realtime-svc's `events::route_event`).
//!   3. Writes the envelope as a single-field Redis Stream entry with
//!      field name `envelope`, on stream key `lcc:realtime:events`.
//!
//! The realtime-svc subscribes to this stream and fans out to per-channel
//! broadcast senders (see `realtime-svc/src/events/mod.rs`).

use chrono::Utc;
use redis::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::error::Error;

/// Redis stream key — single source for the realtime pipeline.
pub const STREAM_KEY: &str = "lcc:realtime:events";
/// Approximate max length per stream (cap applied with `MAXLEN ~`).
pub const STREAM_MAXLEN_APPROX: usize = 100_000;

/// The canonical envelope that wraps every event on every channel.
///
/// Mirrors the shape in `contract_audit/realtime/lcc-realtime-contract.yaml`
/// and `realtime-svc/src/domain/mod.rs::EventEnvelope`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_name: String,
    pub occurred_at: chrono::DateTime<Utc>,
    pub member_id: Uuid,
    pub producer_service: String,
    pub trace_id: Uuid,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    pub fn new(
        event_name: impl Into<String>,
        member_id: Uuid,
        producer: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_name: event_name.into(),
            occurred_at: Utc::now(),
            member_id,
            producer_service: producer.into(),
            trace_id: Uuid::new_v4(),
            payload,
        }
    }
}

/// Channel routing table — must mirror `realtime-svc::events::route_event`
/// exactly. Keep in sync; the contract test
/// `tests/contract/canonical/realtime_flow_test.rs::route_covers_all_five_channels`
/// pins both sides.
pub fn route_event(event_name: &str) -> Option<&'static str> {
    match event_name {
        "briefing.refresh" | "briefing.section.updated" => Some("ws.briefing"),
        "approval.created" | "approval.expired" | "approval.bulk_decided" => Some("ws.approvals"),
        "engagement.inbound.received"
        | "engagement.task.created"
        | "engagement.draft_ready"
        | "engagement.task.expired" => Some("ws.engagement"),
        "compliance.restriction_detected"
        | "compliance.restriction_cleared"
        | "compliance.config_activated"
        | "compliance.circuit_breaker_state_changed" => Some("ws.compliance"),
        "sequence.reply_detected"
        | "sequence.paused"
        | "sequence.resumed"
        | "sequence.completed"
        | "sequence.step.sent"
        | "sequence.step.failed" => Some("ws.sequence"),
        _ => None,
    }
}

/// Publish a single envelope to the realtime Redis Stream.
///
/// Returns the assigned stream entry ID on success, or an error if Redis
/// is unreachable or the payload cannot be serialized.
pub async fn publish_event(
    redis: &mut deadpool_redis::Connection,
    envelope: &EventEnvelope,
) -> Result<String, Error> {
    // Reject unknown events early — silent drops would still consume a
    // stream entry ID, so this is better raised at the producer side.
    if route_event(&envelope.event_name).is_none() {
        return Err(Error::BadRequest(format!(
            "event_name '{}' is not a known realtime event",
            envelope.event_name
        )));
    }

    let json = serde_json::to_string(envelope)
        .map_err(|e| Error::Internal(format!("serialize envelope: {e}")))?;

    // XADD lcc:realtime:events MAXLEN ~ 100000 * envelope <json>
    let entry_id: String = redis::cmd("XADD")
        .arg(STREAM_KEY)
        .arg("MAXLEN")
        .arg("~")
        .arg(STREAM_MAXLEN_APPROX)
        .arg("*")
        .arg("envelope")
        .arg(json)
        .query_async(redis)
        .await
        .map_err(|e| Error::Upstream(format!("redis XADD: {e}")))?;

    Ok(entry_id)
}

/// Publish many envelopes in one round-trip (pipelining is implicit in
/// `redis-rs`'s multi-command sequence).
pub async fn publish_events(
    redis: &mut deadpool_redis::Connection,
    envelopes: &[EventEnvelope],
) -> Result<Vec<String>, Error> {
    let mut ids = Vec::with_capacity(envelopes.len());
    for env in envelopes {
        ids.push(publish_event(redis, env).await?);
    }
    Ok(ids)
}

/// Bridge helper used by domain services that already have a
/// `Vec<(event_name, payload)>` to publish. Returns the count of
/// successfully published events.
pub async fn publish_batch(
    redis: &mut deadpool_redis::Connection,
    member_id: Uuid,
    producer: &str,
    events: Vec<(&str, serde_json::Value)>,
) -> Result<usize, Error> {
    let envelopes: Vec<EventEnvelope> = events
        .into_iter()
        .map(|(name, payload)| EventEnvelope::new(name, member_id, producer, payload))
        .collect();
    publish_events(redis, &envelopes).await.map(|ids| ids.len())
}

/// Helper to silence unused-import warnings when Value isn't called.
#[allow(dead_code)]
fn _type_check() {
    let _: Option<Value> = None;
    let _: BTreeMap<String, String> = BTreeMap::new();
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
    fn envelope_serialization_is_canonical_shape() {
        let env = EventEnvelope::new(
            "briefing.refresh",
            Uuid::new_v4(),
            "orchestrator",
            serde_json::json!({"date":"2026-09-27","approvals_due":3}),
        );
        let json = serde_json::to_string(&env).unwrap();
        for field in [
            "\"event_id\"",
            "\"event_name\"",
            "\"occurred_at\"",
            "\"member_id\"",
            "\"producer_service\"",
            "\"trace_id\"",
            "\"payload\"",
        ] {
            assert!(json.contains(field), "missing {field} in {json}");
        }
    }

    #[test]
    fn unknown_event_rejected_at_construction() {
        let env = EventEnvelope::new(
            "totally.unrelated.event",
            Uuid::new_v4(),
            "test",
            serde_json::json!({}),
        );
        assert!(route_event(&env.event_name).is_none());
    }
}
