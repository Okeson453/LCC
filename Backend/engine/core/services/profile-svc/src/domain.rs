//! Profile domain types.
//!
//! Per Backend Design Concept §5: profile-svc owns the canonical profile
//! snapshots, edit-drafts, and consent records. KB references attach to
//! profile records via `lcc.profile_kb_refs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub id: Uuid,
    pub member_id: Uuid,
    pub version: i32,
    pub headline: String,
    pub summary: String,
    pub skills: Vec<String>,
    pub experiences: Vec<Experience>,
    pub education: Vec<Education>,
    pub kb_ref_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: Uuid,
    pub title: String,
    pub company: String,
    pub started_at: chrono::NaiveDate,
    pub ended_at: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub kb_ref_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub id: Uuid,
    pub institution: String,
    pub degree: Option<String>,
    pub started_at: Option<chrono::NaiveDate>,
    pub ended_at: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEditDraft {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub member_id: Uuid,
    pub proposed_fields: serde_json::Value,
    pub status: EditDraftStatus,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditDraftStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl EditDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: Uuid,
    pub member_id: Uuid,
    pub consent_kind: ConsentKind,
    pub granted: bool,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentKind {
    LinkedinAutomation,
    DataExport,
    KbPersonalization,
}

impl ConsentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinkedinAutomation => "linkedin_automation",
            Self::DataExport => "data_export",
            Self::KbPersonalization => "kb_personalization",
        }
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn snapshot_round_trips() {
        let s = ProfileSnapshot {
            id: Uuid::new_v4(),
            member_id: Uuid::new_v4(),
            version: 3,
            headline: "Staff Engineer".into(),
            summary: "Backend".into(),
            skills: vec!["Rust".into()],
            experiences: vec![],
            education: vec![],
            kb_ref_ids: vec![],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ProfileSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 3);
    }

    #[test]
    fn edit_draft_status_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&EditDraftStatus::Pending).unwrap(),
            "\"pending\""
        );
    }
}
