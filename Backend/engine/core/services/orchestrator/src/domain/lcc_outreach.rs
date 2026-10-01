//! Sequence rows projected into the briefing ("what is due to go out").

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An active sequence with at least one step due to send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceSummary {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub current_step: i32,
    pub last_step_sent_at: Option<DateTime<Utc>>,
    pub status: String,
}
