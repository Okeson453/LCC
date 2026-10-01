//! Tests for the compliance action types.

use lcc_compliance::action::{ActionType, RiskTier};

#[test]
fn connection_request_is_high_risk() {
    assert_eq!(ActionType::ConnectionRequest.risk_tier(), RiskTier::High);
}

#[test]
fn dm_is_high_risk() {
    assert_eq!(ActionType::Dm.risk_tier(), RiskTier::High);
}

#[test]
fn post_publish_is_medium_risk() {
    assert_eq!(ActionType::PostPublish.risk_tier(), RiskTier::Medium);
}

#[test]
fn comment_post_is_low_risk() {
    assert_eq!(ActionType::CommentPost.risk_tier(), RiskTier::Low);
}

#[test]
fn like_post_is_low_risk() {
    assert_eq!(ActionType::LikePost.risk_tier(), RiskTier::Low);
}

#[test]
fn profile_view_is_low_risk() {
    assert_eq!(ActionType::ProfileView.risk_tier(), RiskTier::Low);
}

#[test]
fn job_application_submit_is_high_risk() {
    assert_eq!(ActionType::JobApplicationSubmit.risk_tier(), RiskTier::High);
}

#[test]
fn client_proposal_send_is_high_risk() {
    assert_eq!(ActionType::ClientProposalSend.risk_tier(), RiskTier::High);
}

#[test]
fn executive_outreach_is_high_risk() {
    assert_eq!(ActionType::ExecutiveOutreach.risk_tier(), RiskTier::High);
}

#[test]
fn sequence_step_send_is_medium_risk() {
    assert_eq!(ActionType::SequenceStepSend.risk_tier(), RiskTier::Medium);
}

#[test]
fn all_actions_have_risk_tier() {
    // Smoke: every action can produce a risk tier.
    let actions = [
        ActionType::ConnectionRequest,
        ActionType::Dm,
        ActionType::PostPublish,
        ActionType::CommentPost,
        ActionType::LikePost,
        ActionType::ProfileView,
        ActionType::FollowCompany,
        ActionType::JobApplicationSubmit,
        ActionType::ClientProposalSend,
        ActionType::ExecutiveOutreach,
        ActionType::SequenceStepSend,
    ];
    for a in actions {
        let _ = a.risk_tier();
    }
}
