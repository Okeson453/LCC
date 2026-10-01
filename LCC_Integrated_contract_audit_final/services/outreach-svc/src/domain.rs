//! Outreach-svc domain: sequences, steps, templates.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub id: Uuid,
    pub member_id: Uuid,
    pub contact_id: Uuid,
    pub template_id: Option<Uuid>,
    pub status: SequenceStatus,
    pub current_step: i32,
    pub paused_reason: Option<String>,
    pub last_step_sent_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

impl SequenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStep {
    pub id: Uuid,
    pub sequence_id: Uuid,
    pub step_index: i32,
    pub kind: StepKind,
    pub subject: Option<String>,
    pub body: String,
    pub rendered_body_hash: Option<String>,
    pub contact_id: Uuid,
    pub scheduled_at: Option<NaiveDateTime>,
    pub sent_at: Option<DateTime<Utc>>,
    pub response_received_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    ConnectionRequest,
    Message,
    InMail,
    CommentNudge,
    VoiceNote,
}

impl StepKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConnectionRequest => "connection_request",
            Self::Message => "message",
            Self::InMail => "inmail",
            Self::CommentNudge => "comment_nudge",
            Self::VoiceNote => "voice_note",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub member_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<TemplateStep>,
    pub is_published: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStep {
    pub kind: StepKind,
    pub subject: Option<String>,
    pub body: String,
    pub delay_hours: i32,
}
