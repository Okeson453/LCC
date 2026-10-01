//! Engagement task rows projected into the briefing ("what to reply to").

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A queued or drafted engagement reply awaiting the member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: Uuid,
    pub action_type: String,
    /// Higher sorts first; may be null when unscored.
    pub priority_score: Option<f64>,
    pub due_at: Option<DateTime<Utc>>,
}
