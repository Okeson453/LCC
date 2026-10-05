//! Content-svc domain — content lifecycle with 9 states.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Content state machine.
///
/// Transitions follow Backend Design §10.4.3:
///   idea -> drafted -> in_review -> approved -> scheduled -> published
///   any stage may also transition to: rejected, publish_failed, blocked
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentState {
    Idea,
    Drafted,
    InReview,
    Approved,
    Scheduled,
    Published,
    Rejected,
    PublishFailed,
    Blocked,
}

impl ContentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idea => "idea",
            Self::Drafted => "drafted",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Scheduled => "scheduled",
            Self::Published => "published",
            Self::Rejected => "rejected",
            Self::PublishFailed => "publish_failed",
            Self::Blocked => "blocked",
        }
    }

    /// Whether a state transition is allowed.
    pub fn can_transition_to(self, next: ContentState) -> bool {
        use ContentState::*;
        match (self, next) {
            (Idea, Drafted) => true,
            (Drafted, InReview) => true,
            (InReview, Approved) => true,
            (Approved, Scheduled) => true,
            (Scheduled, Published) => true,
            // Failures are allowed from any "active" state
            (Drafted | InReview | Approved | Scheduled, Rejected) => true,
            (Drafted | InReview | Approved, Blocked) => true,
            (Scheduled, PublishFailed) => true,
            // Recovery: publish_failed can be re-scheduled
            (PublishFailed, Scheduled) => true,
            (PublishFailed, Drafted) => true,
            // Blocked can be unblocked
            (Blocked, Drafted) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: Uuid,
    pub member_id: Uuid,
    pub state: ContentState,
    pub title: String,
    pub body: String,
    pub rendered_body_hash: Option<String>,
    pub kind: ContentKind,
    pub topic: String,
    pub voice_style_kb_id: Option<Uuid>,
    pub pinned_kb_ids: Vec<Uuid>,
    pub metrics: ContentMetrics,
    pub quality_loop_count: i32,
    pub idempotency_key: Option<String>,
    pub expected_version: i32,
    pub version: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Post,
    Article,
    Comment,
    Reply,
    SequenceMessage,
}

impl ContentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Post => "post",
            Self::Article => "article",
            Self::Comment => "comment",
            Self::Reply => "reply",
            Self::SequenceMessage => "sequence_message",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentMetrics {
    pub impressions: Option<i64>,
    pub reactions: Option<i64>,
    pub comments: Option<i64>,
    pub reshares: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub passed: bool,
    /// Raw identifier: `loop` is a Rust keyword. serde still emits the field
    /// as "loop" on the wire, matching the contract.
    pub r#loop: i32,
    pub issues: Vec<String>,
    pub auto_fixes: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub scheduled_at: NaiveDateTime,
    pub slots: Vec<String>,
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn happy_path_transitions() {
        assert!(ContentState::Idea.can_transition_to(ContentState::Drafted));
        assert!(ContentState::Drafted.can_transition_to(ContentState::InReview));
        assert!(ContentState::InReview.can_transition_to(ContentState::Approved));
        assert!(ContentState::Approved.can_transition_to(ContentState::Scheduled));
        assert!(ContentState::Scheduled.can_transition_to(ContentState::Published));
    }

    #[test]
    fn blocked_can_be_unblocked() {
        assert!(ContentState::Blocked.can_transition_to(ContentState::Drafted));
    }

    #[test]
    fn cannot_publish_directly_from_idea() {
        assert!(!ContentState::Idea.can_transition_to(ContentState::Published));
    }

    #[test]
    fn publish_failed_can_reschedule() {
        assert!(ContentState::PublishFailed.can_transition_to(ContentState::Scheduled));
        assert!(ContentState::PublishFailed.can_transition_to(ContentState::Drafted));
    }

    #[test]
    fn state_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ContentState::InReview).unwrap(),
            "\"in_review\""
        );
        assert_eq!(
            serde_json::to_string(&ContentState::PublishFailed).unwrap(),
            "\"publish_failed\""
        );
    }
}
