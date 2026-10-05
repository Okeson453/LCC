//! Approval-svc domain: gating actions, two-reviewer, tiered decisions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub id: Uuid,
    pub member_id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub requested_action: serde_json::Value,
    pub tier: i16,
    pub rule_version: Option<String>,
    pub status: ApprovalStatus,
    pub requested_by: Uuid,
    pub decided_reason: Option<String>,
    pub reviewer_ids: Vec<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestApprovalInput {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub requested_action: serde_json::Value,
    pub tier: i16,
    pub rule_version: Option<String>,
    pub ttl_hours: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDecideInput {
    pub ids: Vec<Uuid>,
    pub decision: ApprovalStatus,
    pub reason: String,
    pub expected_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDecideResult {
    pub approved: usize,
    pub rejected: usize,
    pub conflicts: Vec<Uuid>,
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn status_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ApprovalStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&ApprovalStatus::Expired).unwrap(),
            "\"expired\""
        );
    }
}
