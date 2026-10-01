//! network-crm-svc domain: contacts + companies + interactions + staleness.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub member_id: Uuid,
    pub company_id: Option<Uuid>,
    pub full_name: String,
    pub title: Option<String>,
    pub headline: Option<String>,
    pub linkedin_url: Option<String>,
    pub email: Option<String>,
    pub connection_strength: i16,
    pub last_touched_at: Option<DateTime<Utc>>,
    pub last_interaction_kind: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub stale_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Uuid,
    pub member_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub size_band: Option<String>,
    pub funding_stage: Option<String>,
    pub tech_stack: Vec<String>,
    pub trigger_events: Vec<String>,
    pub public_signals: Vec<String>,
    pub ttl_at: Option<NaiveDate>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Uuid,
    pub member_id: Uuid,
    pub contact_id: Uuid,
    pub kind: InteractionKind,
    pub summary: String,
    pub occurred_at: DateTime<Utc>,
    pub channel: Option<String>,
    pub direction: InteractionDirection,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionKind {
    ManualNote,
    InboundMessage,
    OutboundMessage,
    ConnectionRequest,
    PhoneCall,
    Meeting,
    SequenceStepSent,
    Reaction,
    PublicComment,
    Other,
}

impl InteractionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManualNote => "manual_note",
            Self::InboundMessage => "inbound_message",
            Self::OutboundMessage => "outbound_message",
            Self::ConnectionRequest => "connection_request",
            Self::PhoneCall => "phone_call",
            Self::Meeting => "meeting",
            Self::SequenceStepSent => "sequence_step_sent",
            Self::Reaction => "reaction",
            Self::PublicComment => "public_comment",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InteractionDirection {
    Inbound,
    Outbound,
    Neutral,
}

impl InteractionDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbound => "inbound",
            Self::Outbound => "outbound",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessReport {
    pub member_id: Uuid,
    pub as_of: DateTime<Utc>,
    pub stale_contacts: Vec<StaleContact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleContact {
    pub contact_id: Uuid,
    pub full_name: String,
    pub days_since_touch: i64,
    pub suggested_action: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn interaction_kind_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&InteractionKind::InboundMessage).unwrap(),
            "\"inbound_message\""
        );
        assert_eq!(
            serde_json::to_string(&InteractionKind::SequenceStepSent).unwrap(),
            "\"sequence_step_sent\""
        );
    }
}
