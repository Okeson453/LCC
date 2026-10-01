//! Track router — Track A (official API) vs. Track B (browser-assist).
//!
//! ```math
//! \text{track}(action) = A if action.type ∈ API_SUPPORTED_SET, else B
//! ```
//!
//! Source §37, Backend Design Concept §38.

use lcc_compliance::action::ActionType;
use lcc_compliance::config::ComplianceConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Track {
    TrackA,
    TrackB,
}

impl Track {
    pub fn as_str(self) -> &'static str {
        match self {
            Track::TrackA => "A",
            Track::TrackB => "B",
        }
    }
}

/// Route an action to Track A or Track B based on the action type and the
/// active compliance config's API_SUPPORTED_SET.
///
/// Per Backend Design Concept §37:
/// - Track A: organization-page post publish, OAuth profile read, jobs data read.
/// - Track B: personal-profile posting, connection requests, DMs, most search.
pub fn route(action_type: ActionType, config: &ComplianceConfig) -> Track {
    use ActionType::*;

    // Hard-coded initial set (matches Backend Design Concept §37).
    let in_api_set: bool = matches!(
        action_type,
        // Track A actions
        PostPublish  // organization-page only — at integration time, check author kind
        | KbRecordCreate  // NOT actually LinkedIn API — but RAG retrieval; this maps to None
    );

    // Action types that always go to Track B (human-assist).
    let track_b_explicit = matches!(
        action_type,
        ConnectionRequest
            | DirectMessage
            | SequenceStepSend
            | JobApplicationSubmit
            | ClientProposalSend
            | ExecutiveOutreach
            | ProfileEditSubmit
    );

    if track_b_explicit {
        Track::TrackB
    } else if in_api_set && config.api_supported_set().map(|set| {
        set.iter().any(|s| s == "organization_page_post_publish")
    }).unwrap_or(false) {
        Track::TrackA
    } else {
        Track::TrackB
    }
}

/// Override route for personal-profile posting — even if PostPublish is in
/// the API_SUPPORTED_SET, personal-profile posting is not supported by the
/// official API. The caller must check `is_organization_page` and downgrade
/// to Track B for personal profiles.
pub fn route_with_post_target(action_type: ActionType, is_organization: bool, config: &ComplianceConfig) -> Track {
    if action_type == ActionType::PostPublish && !is_organization {
        return Track::TrackB;
    }
    route(action_type, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_request_is_track_b() {
        let cfg = ComplianceConfig::default();
        assert_eq!(route(ActionType::ConnectionRequest, &cfg), Track::TrackB);
    }

    #[test]
    fn dm_is_track_b() {
        let cfg = ComplianceConfig::default();
        assert_eq!(route(ActionType::DirectMessage, &cfg), Track::TrackB);
    }

    #[test]
    fn org_post_is_track_a() {
        let cfg = ComplianceConfig::default();
        assert_eq!(
            route_with_post_target(ActionType::PostPublish, true, &cfg),
            Track::TrackA
        );
    }

    #[test]
    fn personal_post_is_track_b_even_with_api_set() {
        let cfg = ComplianceConfig::default();
        assert_eq!(
            route_with_post_target(ActionType::PostPublish, false, &cfg),
            Track::TrackB
        );
    }

    #[test]
    fn tier1_draft_actions_route_to_nothing() {
        // Drafting actions are internal; they don't need Track A/B routing.
        // The router returns TrackB as default; the integration-gateway
        // ignores Track for non-executable actions.
        let cfg = ComplianceConfig::default();
        assert_eq!(route(ActionType::ContentDraft, &cfg), Track::TrackB);
    }
}
