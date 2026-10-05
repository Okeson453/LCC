//! Event envelope — wraps every async fan-out payload.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::topics::Topic;

/// EventHeader — the standard envelope fields for every event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventHeader {
    pub event_id: Uuid, // UUIDv7 — primary dedup key
    pub topic: Topic,
    pub trace_id: String,
    pub producer_service: String,
    pub occurred_at: DateTime<Utc>,
    pub member_id: Option<String>, // RLS scope; None for system events
    pub idempotency_key: String,
    pub schema_version: u32,
}

/// EventPayload — typed envelope for each event kind.
///
/// Uses serde tag = "type" for JSON-friendly discrimination. Producers
/// serialize their typed payload; consumers deserialize based on the
/// header's `topic` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    MemberCreated(MemberCreatedEvent),
    ContentItemApproved(ContentItemApprovedEvent),
    ContentItemStateChanged(ContentItemStateChangedEvent),
    EngagementInboundReceived(EngagementInboundReceivedEvent),
    OpportunityDiscovered(OpportunityDiscoveredEvent),
    SequenceReplyDetected(SequenceReplyDetectedEvent),
    SequenceStepDue(SequenceStepDueEvent),
    ComplianceConfigActivated(ComplianceConfigActivatedEvent),
    ComplianceRestrictionDetected(ComplianceRestrictionDetectedEvent),
    AuditEvent(AuditEvent),
    ApprovalDecided(ApprovalDecidedEvent),
    KbRecordCreated(KbRecordCreatedEvent),
}

/// Envelope — the full event published to the bus.
///
/// Serialises as `{ "header": {...}, "payload": {...} }`, which is what
/// `schemas/events/*.schema.json` requires (`required: [header, payload]`,
/// `additionalProperties: false`). It was previously flattened into a single
/// object, which failed that schema and — because `member_id` exists in both
/// the header and several payloads — emitted a duplicate JSON key, leaving
/// deserialisation order-dependent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub header: EventHeader,
    pub payload: EventPayload,
}

impl Envelope {
    pub fn new(
        topic: Topic,
        producer_service: impl Into<String>,
        trace_id: impl Into<String>,
        payload: EventPayload,
    ) -> Self {
        let event_id = Uuid::now_v7();
        let idempotency_key = format!("{}:{}", topic, event_id);
        Self {
            header: EventHeader {
                event_id,
                topic,
                trace_id: trace_id.into(),
                producer_service: producer_service.into(),
                occurred_at: Utc::now(),
                member_id: None,
                idempotency_key,
                schema_version: 1,
            },
            payload,
        }
    }

    pub fn with_member(mut self, member_id: impl Into<String>) -> Self {
        self.header.member_id = Some(member_id.into());
        self
    }
}

// --- Concrete event payload types ---
// These mirror the proto payloads in `proto/lcc/v1/events/events.proto`.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberCreatedEvent {
    pub member_id: String,
    pub linkedin_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentItemApprovedEvent {
    pub item_id: String,
    pub member_id: String,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentItemStateChangedEvent {
    pub item_id: String,
    pub member_id: String,
    pub from_status: String,
    pub to_status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngagementInboundReceivedEvent {
    pub message_id: String,
    pub member_id: String,
    pub from_contact_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpportunityDiscoveredEvent {
    pub opportunity_id: String,
    pub member_id: String,
    pub opp_type: String,
    pub source: String,
    pub fit_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceReplyDetectedEvent {
    pub sequence_id: String,
    pub member_id: String,
    pub contact_id: String,
    pub inbound_message_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceStepDueEvent {
    pub sequence_id: String,
    pub step_id: String,
    pub member_id: String,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceConfigActivatedEvent {
    pub version_id: String,
    pub version: String,
    pub activated_at: DateTime<Utc>,
    pub activated_by: String,
    pub two_reviewer_signed_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceRestrictionDetectedEvent {
    pub member_id: String,
    pub reason: String,
    pub signal_kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub actor: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalDecidedEvent {
    pub approval_id: String,
    pub member_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub approved: bool,
    pub decided_by: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KbRecordCreatedEvent {
    pub record_id: String,
    pub member_id: String,
    pub category: String,
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_serializes_with_type_tag() {
        let env = Envelope::new(
            Topic::MemberCreated,
            "identity-svc",
            "trace_123",
            EventPayload::MemberCreated(MemberCreatedEvent {
                member_id: "m_001".into(),
                linkedin_id: "li_abc".into(),
                created_at: Utc::now(),
            }),
        )
        .with_member("m_001");

        let json = serde_json::to_value(&env).unwrap();
        // `schemas/events/*.schema.json` requires exactly {header, payload}
        // and forbids anything else at the top level.
        assert_eq!(json["header"]["topic"], "member_created");
        assert_eq!(json["payload"]["type"], "member_created");
        assert_eq!(json["payload"]["member_id"], "m_001");
        let top: Vec<&String> = json.as_object().expect("object").keys().collect();
        assert_eq!(top, vec!["header", "payload"]);

        // The header and the payload both carry `member_id`; nested, that is
        // two distinct values. Flattened it produced a duplicate JSON key and
        // left deserialisation order-dependent.
        assert_eq!(json["header"]["member_id"], "m_001");
        assert_eq!(json["payload"]["member_id"], "m_001");
    }

    #[test]
    fn envelope_roundtrip_json() {
        let env = Envelope::new(
            Topic::AuditEvent,
            "audit-svc",
            "trace_xyz",
            EventPayload::AuditEvent(AuditEvent {
                actor: "system:test".into(),
                action: "test.event".into(),
                resource_type: "test".into(),
                resource_id: Some("res_1".into()),
                outcome: "success".into(),
                reason: None,
                checksum_sha256: "abc123".into(),
            }),
        );
        let s = serde_json::to_string(&env).unwrap();
        let back: Envelope = serde_json::from_str(&s).unwrap();
        assert_eq!(back.header.event_id, env.header.event_id);
    }
}
