//! Engagement-svc domain: inbox + queue + tasks + drafts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementTask {
    pub id: Uuid,
    pub member_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub target_post_id: Option<String>,
    pub action_type: ActionType,
    pub status: TaskStatus,
    pub priority_score: Option<f64>,
    pub due_at: Option<DateTime<Utc>>,
    pub draft: Option<String>,
    pub draft_pins: Vec<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Reply,
    Comment,
    Like,
    Connect,
    Remind,
    Share,
    Publish,
    CustomNote,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reply => "reply",
            Self::Comment => "comment",
            Self::Like => "like",
            Self::Connect => "connect",
            Self::Remind => "remind",
            Self::Share => "share",
            Self::Publish => "publish",
            Self::CustomNote => "custom_note",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Drafted,
    Sent,
    Completed,
    Skipped,
    Expired,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Drafted => "drafted",
            Self::Sent => "sent",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxMessage {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub surface: String,
    pub subject: Option<String>,
    pub preview: String,
    pub unread: bool,
    pub received_at: DateTime<Utc>,
}
