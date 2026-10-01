//! Integration test: end-to-end Compliance Governor pipeline.
//!
//! Exercises the full 8-guard pipeline against the in-memory MockComplianceGovernor.
//! Verifies happy path → allow + permit_token, each guard's individual deny case,
//! and the order-of-evaluation invariant (axiom 3: partial-pass = deny).

#[path = "common/harness.rs"]
mod harness;

use harness::{ActionType, MockComplianceGovernor};
use uuid::Uuid;

fn request(action: ActionType, member_id: Uuid, kb_refs: usize) -> harness::GovernorRequest {
    harness::GovernorRequest {
        member_id,
        action_type: action,
        target_kind: "post".into(),
        target_id: Some(format!("target-{member_id}")),
        context: serde_json::json!({ "kb_refs": vec!["r1"; kb_refs] }),
    }
}

#[test]
fn happy_path_post_publish_allowed() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    let req = request(ActionType::PostPublish, member, 2);
    let decision = gov.evaluate(&req);
    assert_eq!(decision.decision, "allow", "happy path must allow");
    assert!(decision.permit_token.is_some(), "allow must mint a permit_token");
    assert_eq!(decision.guards_passed.len(), 8, "all 8 guards must pass");
    assert!(decision.guards_failed.is_empty());
}

#[test]
fn low_grounding_denies_post_publish() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    let req = request(ActionType::PostPublish, member, 0); // 0 KB refs
    let decision = gov.evaluate(&req);
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["grounding"]);
    assert_eq!(decision.guards_passed.len(), 4, "guards before grounding must pass");
    assert!(decision.permit_token.is_none());
}

#[test]
fn daily_cap_deny_stops_at_first_guard() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    // connection_request cap is 25.
    for _ in 0..25 {
        let _ = gov.evaluate(&request(ActionType::ConnectionRequest, member, 1));
    }
    let decision = gov.evaluate(&request(ActionType::ConnectionRequest, member, 1));
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["daily_cap"]);
    assert_eq!(decision.guards_passed.len(), 0, "no guards pass before daily_cap");
}

#[test]
fn restriction_blocks_all_actions() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    gov.set_restriction(member, true);
    let decision = gov.evaluate(&request(ActionType::PostPublish, member, 5));
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["account_health"]);
}

#[test]
fn session_pacing_deny_after_max_in_window() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    // session_pacing_max is 5 per 600s window.
    for _ in 0..5 {
        let r = gov.evaluate(&request(ActionType::PostPublish, member, 1));
        assert_eq!(r.decision, "allow", "first 5 must allow");
    }
    let decision = gov.evaluate(&request(ActionType::PostPublish, member, 1));
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["session_pacing"]);
}

#[test]
fn each_risk_tier_maps_correctly() {
    use harness::RiskTier;
    assert_eq!(ActionType::ConnectionRequest.risk_tier(), RiskTier::High);
    assert_eq!(ActionType::Dm.risk_tier(), RiskTier::High);
    assert_eq!(ActionType::JobApplicationSubmit.risk_tier(), RiskTier::High);
    assert_eq!(ActionType::ClientProposalSend.risk_tier(), RiskTier::High);
    assert_eq!(ActionType::ExecutiveOutreach.risk_tier(), RiskTier::High);
    assert_eq!(ActionType::PostPublish.risk_tier(), RiskTier::Medium);
    assert_eq!(ActionType::SequenceStepSend.risk_tier(), RiskTier::Medium);
    assert_eq!(ActionType::CommentPost.risk_tier(), RiskTier::Low);
    assert_eq!(ActionType::LikePost.risk_tier(), RiskTier::Low);
    assert_eq!(ActionType::ProfileView.risk_tier(), RiskTier::Low);
    assert_eq!(ActionType::FollowCompany.risk_tier(), RiskTier::Low);
}
