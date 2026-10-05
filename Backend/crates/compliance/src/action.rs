//! ActionType and RiskTier enums.
//!
//! These are the *only* canonical classifications — every candidate action
//! presented to the Compliance Governor must declare both. Any unclassified
//! action type is rejected by guard 4 (account-health tier check) before any
//! other evaluation occurs.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ActionType — taxonomy of every externally-visible or reputationally-relevant
/// action the system can take. Must match `lcc.v1.compliance.ActionType` in proto.
///
/// Mirrors `proto/lcc/v1/compliance/governor.proto::ActionType` and is the
/// single source of truth used by the Rust Core Engine (Python Intelligence
/// Engine has a `packages/compliance-types` mirror).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Tier 1 — drafting / editing (auto-execute)
    ProfileEditDraft,
    ContentDraft,
    KbRecordCreate,
    SequenceDraft,
    OpportunityDiscover,
    ResearchScan,
    VoiceTrain,

    // Tier 2 — light engagement / publish (H_c ≥ 0.3)
    Comment,
    Like,
    PostPublish,
    ProfileEditSubmit,

    // Tier 3 — network growth (H_c ≥ 0.4)
    ConnectionRequest,

    // Tier 4 — 1:1 outreach (H_c ≥ 0.5)
    DirectMessage,
    SequenceStepSend,

    // Tier 5 — high stakes (H_c ≥ 0.5)
    JobApplicationSubmit,
    ClientProposalSend,
    ExecutiveOutreach,
}

impl ActionType {
    /// Returns the RiskTier for this ActionType.
    pub fn risk_tier(self) -> RiskTier {
        match self {
            ActionType::ProfileEditDraft
            | ActionType::ContentDraft
            | ActionType::KbRecordCreate
            | ActionType::SequenceDraft
            | ActionType::OpportunityDiscover
            | ActionType::ResearchScan
            | ActionType::VoiceTrain => RiskTier::Tier1DraftOrEdit,

            ActionType::Comment
            | ActionType::Like
            | ActionType::PostPublish
            | ActionType::ProfileEditSubmit => RiskTier::Tier2LightEngagement,

            ActionType::ConnectionRequest => RiskTier::Tier3NetworkGrowth,

            ActionType::DirectMessage | ActionType::SequenceStepSend => {
                RiskTier::Tier4OneToOneOutreach
            }

            ActionType::JobApplicationSubmit
            | ActionType::ClientProposalSend
            | ActionType::ExecutiveOutreach => RiskTier::Tier5HighStakes,
        }
    }

    /// Returns the `CAP_base` for this ActionType when `H_c ≥ 0.7` (Scenario B).
    /// Source: Technical Design Spec §12.
    pub fn cap_base(self) -> u32 {
        match self {
            ActionType::ConnectionRequest => 18,
            ActionType::DirectMessage | ActionType::SequenceStepSend => 25,
            ActionType::Comment => 15,
            ActionType::Like => 40,
            ActionType::ProfileEditSubmit => 3,
            ActionType::PostPublish => 3,
            // Tier-1 actions do not consume daily quota (they're auto).
            _ => 0,
        }
    }

    /// Returns the `CAP_base` for the warm-up floor (H_c < 0.3, days 1–28).
    pub fn warm_up_cap(self) -> u32 {
        match self {
            ActionType::ConnectionRequest => 5,
            ActionType::DirectMessage | ActionType::SequenceStepSend => 6,
            ActionType::Comment => 4,
            ActionType::Like => 10,
            ActionType::ProfileEditSubmit => 1,
            ActionType::PostPublish => 1,
            _ => 0,
        }
    }

    /// Minimum inter-action spacing in seconds (Source: §12).
    pub fn min_spacing_seconds(self) -> u64 {
        match self {
            ActionType::ConnectionRequest => 90,
            ActionType::DirectMessage | ActionType::SequenceStepSend => 60,
            ActionType::Comment => 45,
            ActionType::Like => 15,
            // Tier-1 actions do not require spacing.
            _ => 0,
        }
    }

    /// Whether this action type carries an external artifact (post, message, edit).
    /// External-facing artifacts must cite ≥1 KB record (axiom 5, guard 5).
    pub fn requires_grounding(self) -> bool {
        matches!(
            self,
            ActionType::PostPublish
                | ActionType::Comment
                | ActionType::DirectMessage
                | ActionType::SequenceStepSend
                | ActionType::ConnectionRequest
                | ActionType::ProfileEditSubmit
                | ActionType::ClientProposalSend
                | ActionType::ExecutiveOutreach
        )
    }

    /// Whether this action type defaults to auto-execute (Tier 1 only).
    pub fn auto_execute_by_default(self) -> bool {
        matches!(self.risk_tier(), RiskTier::Tier1DraftOrEdit)
    }

    /// All values — for config validation and proto roundtripping.
    pub const ALL: &'static [ActionType] = &[
        ActionType::ProfileEditDraft,
        ActionType::ContentDraft,
        ActionType::KbRecordCreate,
        ActionType::SequenceDraft,
        ActionType::OpportunityDiscover,
        ActionType::ResearchScan,
        ActionType::VoiceTrain,
        ActionType::Comment,
        ActionType::Like,
        ActionType::PostPublish,
        ActionType::ProfileEditSubmit,
        ActionType::ConnectionRequest,
        ActionType::DirectMessage,
        ActionType::SequenceStepSend,
        ActionType::JobApplicationSubmit,
        ActionType::ClientProposalSend,
        ActionType::ExecutiveOutreach,
    ];
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ActionType {
    /// Borrowed string form. Mirrors `Display`, useful where a `String` is
    /// unwanted (e.g., builders and permit-claim construction).
    pub fn as_str(self) -> &'static str {
        match self {
            ActionType::ProfileEditDraft => "profile_edit_draft",
            ActionType::ContentDraft => "content_draft",
            ActionType::KbRecordCreate => "kb_record_create",
            ActionType::SequenceDraft => "sequence_draft",
            ActionType::OpportunityDiscover => "opportunity_discover",
            ActionType::ResearchScan => "research_scan",
            ActionType::VoiceTrain => "voice_train",
            ActionType::Comment => "comment",
            ActionType::Like => "like",
            ActionType::PostPublish => "post_publish",
            ActionType::ProfileEditSubmit => "profile_edit_submit",
            ActionType::ConnectionRequest => "connection_request",
            ActionType::DirectMessage => "direct_message",
            ActionType::SequenceStepSend => "sequence_step_send",
            ActionType::JobApplicationSubmit => "job_application_submit",
            ActionType::ClientProposalSend => "client_proposal_send",
            ActionType::ExecutiveOutreach => "executive_outreach",
        }
    }
}

/// RiskTier (1..5) — required minimum H_c to permit the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Tier1DraftOrEdit = 1,
    Tier2LightEngagement = 2,
    Tier3NetworkGrowth = 3,
    Tier4OneToOneOutreach = 4,
    Tier5HighStakes = 5,
}

impl RiskTier {
    pub fn minimum_h_c(self) -> f64 {
        match self {
            // Tier 1 has no H_c requirement (drafting is always allowed).
            RiskTier::Tier1DraftOrEdit => 0.0,
            RiskTier::Tier2LightEngagement => 0.3,
            RiskTier::Tier3NetworkGrowth => 0.4,
            RiskTier::Tier4OneToOneOutreach => 0.5,
            RiskTier::Tier5HighStakes => 0.5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RiskTier::Tier1DraftOrEdit => "tier_1_draft_or_edit",
            RiskTier::Tier2LightEngagement => "tier_2_light_engagement",
            RiskTier::Tier3NetworkGrowth => "tier_3_network_growth",
            RiskTier::Tier4OneToOneOutreach => "tier_4_one_to_one_outreach",
            RiskTier::Tier5HighStakes => "tier_5_high_stakes",
        }
    }
}

impl fmt::Display for RiskTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_to_tier_mapping() {
        assert_eq!(
            ActionType::ConnectionRequest.risk_tier(),
            RiskTier::Tier3NetworkGrowth
        );
        assert_eq!(
            ActionType::PostPublish.risk_tier(),
            RiskTier::Tier2LightEngagement
        );
        assert_eq!(
            ActionType::JobApplicationSubmit.risk_tier(),
            RiskTier::Tier5HighStakes
        );
    }

    #[test]
    fn tier_minimums() {
        assert_eq!(RiskTier::Tier1DraftOrEdit.minimum_h_c(), 0.0);
        assert_eq!(RiskTier::Tier3NetworkGrowth.minimum_h_c(), 0.4);
        assert_eq!(RiskTier::Tier5HighStakes.minimum_h_c(), 0.5);
    }

    #[test]
    fn caps_match_spec() {
        assert_eq!(ActionType::ConnectionRequest.cap_base(), 18);
        assert_eq!(ActionType::DirectMessage.cap_base(), 25);
        assert_eq!(ActionType::Comment.cap_base(), 15);
        assert_eq!(ActionType::Like.cap_base(), 40);
        assert_eq!(ActionType::ConnectionRequest.warm_up_cap(), 5);
        assert_eq!(ActionType::DirectMessage.warm_up_cap(), 6);
    }

    #[test]
    fn spacing_match_spec() {
        assert_eq!(ActionType::ConnectionRequest.min_spacing_seconds(), 90);
        assert_eq!(ActionType::Like.min_spacing_seconds(), 15);
    }

    #[test]
    fn grounding_required() {
        assert!(ActionType::PostPublish.requires_grounding());
        assert!(ActionType::ConnectionRequest.requires_grounding());
        assert!(!ActionType::KbRecordCreate.requires_grounding());
    }

    #[test]
    fn auto_execute_only_tier1() {
        assert!(ActionType::ProfileEditDraft.auto_execute_by_default());
        assert!(ActionType::ContentDraft.auto_execute_by_default());
        assert!(!ActionType::ConnectionRequest.auto_execute_by_default());
    }

    #[test]
    fn display_roundtrip() {
        for a in ActionType::ALL {
            let s = a.to_string();
            assert!(!s.is_empty());
        }
    }
}
