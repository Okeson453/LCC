//! Topic constants — single source of truth for every event topic in the system.
//!
//! Per Backend Monorepo Layout §29, every topic must also have a JSON-Schema
//! in `schemas/events/<topic>.schema.json`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    // Member lifecycle
    MemberCreated,
    MemberDeactivated,
    OAuthTokenRefreshFailed,

    // Profile
    ProfileSnapshotCreated,
    ProfileAuditCompleted,

    // KB / Voice
    KbRecordCreated,
    KbRecordUpdated,
    KbRecordDeleted,
    VoiceSampleAdded,

    // Content
    ContentItemStateChanged,
    ContentItemApproved,

    // Engagement
    EngagementInboundReceived,
    EngagementReplyDrafted,
    SequenceReplyDetected,
    SequenceStepDue,
    SequenceStateChanged,
    SequenceStepSent,

    // Opportunity
    OpportunityDiscovered,
    OpportunityQualified,
    OpportunityFunnelChanged,

    // Approval
    ApprovalDecided,
    ApprovalExpired,

    // Compliance
    ComplianceConfigActivated,
    ComplianceRestrictionDetected,
    ComplianceRestrictionCleared,

    // Universal audit
    AuditEvent,
}

impl Topic {
    /// String identifier (matches `schemas/events/<name>.schema.json`).
    pub fn as_str(self) -> &'static str {
        match self {
            Topic::MemberCreated => "member.created",
            Topic::MemberDeactivated => "member.deactivated",
            Topic::OAuthTokenRefreshFailed => "oauth.token_refresh_failed",
            Topic::ProfileSnapshotCreated => "profile.snapshot.created",
            Topic::ProfileAuditCompleted => "profile.audit.completed",
            Topic::KbRecordCreated => "kb.record.created",
            Topic::KbRecordUpdated => "kb.record.updated",
            Topic::KbRecordDeleted => "kb.record.deleted",
            Topic::VoiceSampleAdded => "voice.sample.added",
            Topic::ContentItemStateChanged => "content.item.state_changed",
            Topic::ContentItemApproved => "content.item.approved",
            Topic::EngagementInboundReceived => "engagement.inbound.received",
            Topic::EngagementReplyDrafted => "engagement.reply_drafted",
            Topic::SequenceReplyDetected => "sequence.reply_detected",
            Topic::SequenceStepDue => "sequence.step.due",
            Topic::SequenceStateChanged => "sequence.state_changed",
            Topic::SequenceStepSent => "sequence.step.sent",
            Topic::OpportunityDiscovered => "opportunity.discovered",
            Topic::OpportunityQualified => "opportunity.qualified",
            Topic::OpportunityFunnelChanged => "opportunity.funnel_changed",
            Topic::ApprovalDecided => "approval.decided",
            Topic::ApprovalExpired => "approval.expired",
            Topic::ComplianceConfigActivated => "compliance.config_activated",
            Topic::ComplianceRestrictionDetected => "compliance.restriction_detected",
            Topic::ComplianceRestrictionCleared => "compliance.restriction_cleared",
            Topic::AuditEvent => "audit.event",
        }
    }

    /// Owner service — who emits this topic. Used for routing/audit.
    pub fn owner(self) -> &'static str {
        match self {
            Topic::MemberCreated | Topic::MemberDeactivated | Topic::OAuthTokenRefreshFailed => {
                "identity-svc"
            }
            Topic::ProfileSnapshotCreated | Topic::ProfileAuditCompleted => "profile-svc",
            Topic::KbRecordCreated | Topic::KbRecordUpdated | Topic::KbRecordDeleted => "kb-svc",
            Topic::VoiceSampleAdded => "voice-intel",
            Topic::ContentItemStateChanged | Topic::ContentItemApproved => "content-svc",
            Topic::EngagementInboundReceived
            | Topic::EngagementReplyDrafted
            | Topic::SequenceReplyDetected
            | Topic::SequenceStepDue
            | Topic::SequenceStateChanged
            | Topic::SequenceStepSent => "engagement-svc|outreach-svc",
            Topic::OpportunityDiscovered
            | Topic::OpportunityQualified
            | Topic::OpportunityFunnelChanged => "opportunity-svc",
            Topic::ApprovalDecided | Topic::ApprovalExpired => "approval-svc",
            Topic::ComplianceConfigActivated
            | Topic::ComplianceRestrictionDetected
            | Topic::ComplianceRestrictionCleared => "compliance-governor",
            Topic::AuditEvent => "any",
        }
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_strings_are_dot_separated() {
        for s in [
            Topic::MemberCreated.as_str(),
            Topic::ContentItemApproved.as_str(),
            Topic::ComplianceRestrictionDetected.as_str(),
        ] {
            assert!(s.contains('.'), "topic '{s}' must be dot-separated");
        }
    }

    #[test]
    fn owner_always_present() {
        for t in [
            Topic::MemberCreated,
            Topic::ContentItemApproved,
            Topic::ComplianceConfigActivated,
            Topic::AuditEvent,
        ] {
            assert!(!t.owner().is_empty());
        }
    }
}
