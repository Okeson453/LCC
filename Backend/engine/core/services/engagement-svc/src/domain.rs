//! Engagement-svc domain: engagement tasks (queue) + the unified inbox.
//!
//! # Where every field comes from
//!
//! The authoritative shape is Backend Design Concept §11.12 (`engagement_task`),
//! mirrored by `Contract/openapi/lcc-api-canonical.yaml#/components/schemas/
//! EngagementTask` and by `Backend/proto/lcc/v1/engagement/engagement.proto`.
//! All three agree, and the live table is `lcc.engagement_replies`
//! (`Contract/docs/endpoint_contract_matrix.md:343`:
//! "`EngagementTask` | `lcc.engagement_replies`").
//!
//! Migration 0021 added the three columns §11.12 declares that the table was
//! missing (`action_type`, `draft_body`, `contact_id`); see
//! `Backend/schemas/migrations/0021_engagement_task_alignment.sql` for why each
//! is required by three independent authorities and cannot be derived.
//!
//! Two fields the previous version of this file carried were REMOVED because no
//! authority declares them and no column exists:
//!   * `updated_at` — absent from the contract, from design §11.12 and from the
//!     table. `version` is the optimistic-concurrency column the contract does
//!     declare.
//!   * `draft_pins` — absent from the contract, the design and the proto.
//!     (`lcc.content_items.kb_refs` exists but belongs to the content pipeline,
//!     not to an engagement task, so borrowing it would have been a fiction.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// One row of `lcc.engagement_replies`, shaped as the canonical `EngagementTask`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngagementTask {
    pub id: Uuid,
    pub member_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub target_post_id: Option<String>,
    pub action_type: ActionType,
    pub status: TaskStatus,
    pub priority_score: Option<f64>,
    pub due_at: Option<DateTime<Utc>>,
    /// design §11.12 `draft_body`; contract `EngagementTask.draft_body`.
    pub draft_body: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

/// The outbound action a task asks for.
///
/// Vocabulary is design §11.12's CHECK list, the OpenAPI `EngagementTask
/// .action_type` enum and the proto `EngagementActionType` enum — all three
/// name the same six values, in the same spelling.
///
/// The previous enum here carried `connect`, `remind`, `share`, `publish` and
/// `custom_note`. None of those appears in any of the three authorities, and
/// `lcc.engagement_replies` has no CHECK to catch them, so a write using them
/// would have been stored and returned under a 200. They are gone.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Reply,
    Comment,
    Like,
    Congratulate,
    ConnectionAccept,
    FollowUp,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reply => "reply",
            Self::Comment => "comment",
            Self::Like => "like",
            Self::Congratulate => "congratulate",
            Self::ConnectionAccept => "connection_accept",
            Self::FollowUp => "follow_up",
        }
    }
}

impl FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reply" => Ok(Self::Reply),
            "comment" => Ok(Self::Comment),
            "like" => Ok(Self::Like),
            "congratulate" => Ok(Self::Congratulate),
            "connection_accept" => Ok(Self::ConnectionAccept),
            "follow_up" => Ok(Self::FollowUp),
            other => Err(other.to_string()),
        }
    }
}

/// Lifecycle of an engagement task.
///
/// Vocabulary is design §11.12's CHECK list, the OpenAPI `EngagementTask
/// .status` enum and the proto `EngagementTaskStatus` enum — all three name
/// the same six values.
///
/// The previous enum here carried `Completed` and `Skipped`, and the service
/// wrote `'completed'` and `'skipped'` into the column. Neither word is in the
/// canonical vocabulary: 0007 declares `status TEXT` with no CHECK, so both
/// were silently accepted and then returned under a 200 to a client whose
/// contract enum does not contain them. `complete` now writes `sent` (the
/// canonical terminal state for a reply that went out) and `dismiss` writes
/// `dismissed`. Migration 0021 adds a CHECK so the column itself is fail-closed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Drafted,
    Approved,
    Sent,
    Dismissed,
    Expired,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Drafted => "drafted",
            Self::Approved => "approved",
            Self::Sent => "sent",
            Self::Dismissed => "dismissed",
            Self::Expired => "expired",
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "drafted" => Ok(Self::Drafted),
            "approved" => Ok(Self::Approved),
            "sent" => Ok(Self::Sent),
            "dismissed" => Ok(Self::Dismissed),
            "expired" => Ok(Self::Expired),
            other => Err(other.to_string()),
        }
    }
}

/// One row of `lcc.inbound_messages` (migration 0007), shaped as the canonical
/// `InboxItem`.
///
/// This replaces the previous `InboxMessage`, which read from
/// `lcc.engagement_inbox` — a table that does not exist, appears in neither the
/// OpenAPI contract nor the design docs, and was invented by the service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InboxMessage {
    pub id: Uuid,
    /// `inbound_messages.contact_id` is nullable (0007:11) and is not FK-backed.
    pub contact_id: Option<Uuid>,
    /// `inbound_messages.kind` (the `lcc.inbound_kind` enum), verbatim.
    pub kind: InboundKind,
    /// Derived: `LEFT(body, 280)`. `lcc.inbound_messages` has no `preview`
    /// column; the proto names this field `body_preview` and the contract names
    /// it `preview`, so a truncated rendering of `body` is the intended value.
    pub preview: String,
    /// Derived: `read_at IS NULL`. There is no `unread` boolean in the schema.
    pub unread: bool,
    pub thread_id: Option<String>,
    pub received_at: DateTime<Utc>,
}

/// `lcc.inbound_kind` (migration 0007:6).
///
/// The OpenAPI contract's `InboxItem.kind` enum disagrees with the database:
/// it says `[dm, comment, connection_request, mention, reaction]` where the
/// column says `[comment, dm, mention, connection_request_inbound,
/// post_reaction]`. `connection_request_inbound` / `post_reaction` are the
/// deliberate, collision-avoiding spellings the schema authors chose (the
/// outbound `connection_request` and `post_reaction` actions live in other
/// vocabularies), so the enum is returned as stored and the contract was
/// corrected to match rather than the data being reshaped to fit the contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InboundKind {
    Comment,
    Dm,
    Mention,
    ConnectionRequestInbound,
    PostReaction,
}

impl InboundKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Dm => "dm",
            Self::Mention => "mention",
            Self::ConnectionRequestInbound => "connection_request_inbound",
            Self::PostReaction => "post_reaction",
        }
    }
}

impl FromStr for InboundKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comment" => Ok(Self::Comment),
            "dm" => Ok(Self::Dm),
            "mention" => Ok(Self::Mention),
            "connection_request_inbound" => Ok(Self::ConnectionRequestInbound),
            "post_reaction" => Ok(Self::PostReaction),
            other => Err(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every value the three authorities name, and nothing else. This is the
    /// guard against a future edit re-introducing an invented vocabulary: it
    /// pins the exact set, so an addition fails until the contract, the design
    /// and the proto all carry it.
    #[test]
    fn action_type_vocabulary_is_exactly_the_canonical_six() {
        let all = [
            ActionType::Reply,
            ActionType::Comment,
            ActionType::Like,
            ActionType::Congratulate,
            ActionType::ConnectionAccept,
            ActionType::FollowUp,
        ];
        let rendered: Vec<&str> = all.iter().map(|a| a.as_str()).collect();
        assert_eq!(
            rendered,
            vec![
                "reply",
                "comment",
                "like",
                "congratulate",
                "connection_accept",
                "follow_up"
            ]
        );
        for a in all {
            assert_eq!(ActionType::from_str(a.as_str()).unwrap(), a);
        }
    }

    #[test]
    fn task_status_vocabulary_is_exactly_the_canonical_six() {
        let all = [
            TaskStatus::Queued,
            TaskStatus::Drafted,
            TaskStatus::Approved,
            TaskStatus::Sent,
            TaskStatus::Dismissed,
            TaskStatus::Expired,
        ];
        let rendered: Vec<&str> = all.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            rendered,
            vec![
                "queued",
                "drafted",
                "approved",
                "sent",
                "dismissed",
                "expired"
            ]
        );
        for s in all {
            assert_eq!(TaskStatus::from_str(s.as_str()).unwrap(), s);
        }
    }

    /// The two values the previous implementation wrote into the column. They
    /// are not in design §11.12, the contract enum, or the proto enum, and
    /// migration 0021 now rejects them at the column.
    #[test]
    fn removed_status_values_are_rejected() {
        assert!(TaskStatus::from_str("completed").is_err());
        assert!(TaskStatus::from_str("skipped").is_err());
    }

    /// The five action values the previous implementation accepted. None is in
    /// any authority, and `lcc.engagement_replies.action_type` is CHECK-constrained.
    #[test]
    fn removed_action_values_are_rejected() {
        for gone in ["connect", "remind", "share", "publish", "custom_note"] {
            assert!(
                ActionType::from_str(gone).is_err(),
                "{gone} must not be accepted: it is in no authority"
            );
        }
    }

    /// `lcc.inbound_kind` verbatim (migration 0007:6).
    #[test]
    fn inbound_kind_matches_the_lcc_inbound_kind_enum() {
        let all = [
            InboundKind::Comment,
            InboundKind::Dm,
            InboundKind::Mention,
            InboundKind::ConnectionRequestInbound,
            InboundKind::PostReaction,
        ];
        let rendered: Vec<&str> = all.iter().map(|k| k.as_str()).collect();
        assert_eq!(
            rendered,
            vec![
                "comment",
                "dm",
                "mention",
                "connection_request_inbound",
                "post_reaction"
            ]
        );
        for k in all {
            assert_eq!(InboundKind::from_str(k.as_str()).unwrap(), k);
        }
    }

    #[test]
    fn serde_rendering_is_snake_case() {
        let json = serde_json::to_string(&ActionType::ConnectionAccept).unwrap();
        assert_eq!(json, "\"connection_accept\"");
        let json = serde_json::to_string(&InboundKind::PostReaction).unwrap();
        assert_eq!(json, "\"post_reaction\"");
        let json = serde_json::to_string(&TaskStatus::Dismissed).unwrap();
        assert_eq!(json, "\"dismissed\"");
    }
}
