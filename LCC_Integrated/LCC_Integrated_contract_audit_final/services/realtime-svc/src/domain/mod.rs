//! Realtime domain types.
//!
//! These mirror the schemas defined in
//! `contract_audit/realtime/lcc-realtime-contract.yaml`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Canonical channel ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Briefing,
    Approvals,
    Engagement,
    Compliance,
    Sequence,
}

impl Channel {
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "ws.briefing" => Some(Self::Briefing),
            "ws.approvals" => Some(Self::Approvals),
            "ws.engagement" => Some(Self::Engagement),
            "ws.compliance" => Some(Self::Compliance),
            "ws.sequence" => Some(Self::Sequence),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Briefing => "ws.briefing",
            Self::Approvals => "ws.approvals",
            Self::Engagement => "ws.engagement",
            Self::Compliance => "ws.compliance",
            Self::Sequence => "ws.sequence",
        }
    }

    pub fn allowed_events(&self) -> &'static [&'static str] {
        match self {
            Self::Briefing => &[
                "briefing.refresh",
                "briefing.section.updated",
            ],
            Self::Approvals => &[
                "approval.created",
                "approval.expired",
                "approval.bulk_decided",
            ],
            Self::Engagement => &[
                "engagement.inbound.received",
                "engagement.task.created",
                "engagement.draft_ready",
                "engagement.task.expired",
            ],
            Self::Compliance => &[
                "compliance.restriction_detected",
                "compliance.restriction_cleared",
                "compliance.config_activated",
                "compliance.circuit_breaker_state_changed",
            ],
            Self::Sequence => &[
                "sequence.reply_detected",
                "sequence.paused",
                "sequence.resumed",
                "sequence.completed",
                "sequence.step.sent",
                "sequence.step.failed",
            ],
        }
    }
}

/// The envelope that wraps every event on every channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_name: String,
    pub occurred_at: DateTime<Utc>,
    pub member_id: Uuid,
    pub producer_service: String,
    pub trace_id: String,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Validate that the envelope is shaped correctly and that
    /// `event_name` is allowed for the target channel.
    pub fn validate_for(&self, channel: Channel) -> Result<(), String> {
        if self.event_id.is_nil() {
            return Err("event_id is nil".into());
        }
        if !channel.allowed_events().contains(&self.event_name.as_str()) {
            return Err(format!(
                "event_name '{}' is not allowed on channel '{}'",
                self.event_name,
                channel.id()
            ));
        }
        if self.member_id.is_nil() {
            return Err("member_id is nil".into());
        }
        Ok(())
    }
}
