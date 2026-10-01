//! Approval rows projected into the briefing ("what needs my sign-off").

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A pending approval awaiting the member's decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSummary {
    pub id: Uuid,
    /// `lcc.approvals.resource_type`
    pub resource_type: String,
    /// Read from `requested_action->>'action_type'`
    pub action_type: String,
    /// Risk tier 1-5; tier >= 3 needs a second reviewer.
    pub tier: i16,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
