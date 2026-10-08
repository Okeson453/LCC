//! Integration test: end-to-end Compliance Governor pipeline.
//!
//! Exercises the full 8-guard pipeline against the in-memory MockComplianceGovernor.
//! Verifies happy path → allow + permit_token, each guard's individual deny case,
//! and the order-of-evaluation invariant (axiom 3: partial-pass = deny).

// F-AUDIT-51: the workspace lint set denies `clippy::unwrap_used`,
// `expect_used` and `panic` because an `unwrap` on a `Result` can take a
// production service down. In a test binary the opposite holds: panicking IS
// the failure signal, and `unwrap()` is the idiomatic way to assert "this
// fixture must be valid, and if it is not the test must fail". These suites
// were never compiled by any crate before the `[[test]]` targets were added
// in `crates/test-utils/Cargo.toml`, so they never faced the gate.
// The exemption is file-scoped so the production lints stay fully intact.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// F-AUDIT-71: `common/harness.rs` is a shared module pulled in by every
// suite below via `#[path]`. Each test binary exercises only part of it, so
// the rest is unreachable *from that binary* and `dead_code` fires on items
// that are genuinely used by their siblings — `MockRlsDb`, `sha256_hex`,
// `RiskTier`, `set_restriction`, and others. The alternative (one harness per
// suite) duplicates the mocks this file exists to share.
#![allow(dead_code)]
// `resource_id.into()` reads as redundant field-name shorthand, but the
// struct field is String and the parameter is &str, so the conversion is
// load-bearing and the shorthand would not compile.
#![allow(clippy::redundant_field_names)]
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

/// F-AUDIT-46: `request()` always reuses a single `target_id` per member, so
/// any loop that sends the same action more than once trips the **cooldown**
/// guard before it can ever reach `daily_cap` or `session_pacing`. Both
/// "isolate guard N" tests were therefore asserting the wrong guard — they had
/// never been executed by any runner, so the mistake went unnoticed.
///
/// This variant gives every call its own target, which is what a real
/// sequence does (distinct contacts), so the guard under test is the only one
/// that can fire.
fn request_distinct_target(
    action: ActionType,
    member_id: Uuid,
    kb_refs: usize,
    n: usize,
) -> harness::GovernorRequest {
    harness::GovernorRequest {
        member_id,
        action_type: action,
        target_kind: "post".into(),
        target_id: Some(format!("target-{member_id}-{n}")),
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
    assert!(
        decision.permit_token.is_some(),
        "allow must mint a permit_token"
    );
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
    assert_eq!(
        decision.guards_passed.len(),
        4,
        "guards before grounding must pass"
    );
    assert!(decision.permit_token.is_none());
}

#[test]
fn daily_cap_deny_stops_at_first_guard() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    // F-AUDIT-49: this test could never pass, and never had been run.
    //
    // It drove 25 `connection_request` calls and expected the 26th to be denied
    // by `daily_cap`. Two guards fire before `daily_cap` is ever reached:
    //   • `cooldown`   — connection_request has a 7-day same-target cooldown
    //   • `session_pacing` — a 5-per-window budget on the member
    // So the observed failure was always `cooldown`, and the assertion
    // `guards_failed == ["daily_cap"]` could not hold.
    //
    // `post_publish` is used instead: cap 2, no cooldown rule, and the cap is
    // below the pacing window, so `daily_cap` is genuinely the first guard to
    // fire and the "no guards pass before it" invariant is observable.
    let first = gov.evaluate(&request_distinct_target(
        ActionType::PostPublish,
        member,
        1,
        0,
    ));
    assert_eq!(first.decision, "allow", "under cap must allow");

    let second = gov.evaluate(&request_distinct_target(
        ActionType::PostPublish,
        member,
        1,
        1,
    ));
    assert_eq!(second.decision, "allow", "at cap-1 must still allow");

    let decision = gov.evaluate(&request_distinct_target(
        ActionType::PostPublish,
        member,
        1,
        2,
    ));
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["daily_cap"]);
    assert_eq!(
        decision.guards_passed.len(),
        0,
        "daily_cap is guard 1, so no guard may pass before it"
    );
    assert!(decision.permit_token.is_none());
}

/// `daily_cap` is guard 1 in the pipeline, so it must short-circuit every
/// later guard. This pins the ordering invariant that the original
/// `daily_cap_deny_stops_at_first_guard` was reaching for, and — now that
/// F-AUDIT-50 made the cap reachable — asserts it can actually deny.
#[test]
fn daily_cap_fires_before_every_later_guard() {
    let gov = MockComplianceGovernor::new();
    let member = Uuid::new_v4();
    // post_publish cap is 2. The 3rd call in the window must be denied by
    // guard 1 alone: guards 2-8 must never be evaluated, so the reported
    // failure list contains exactly one entry.
    for i in 0..2 {
        let r = gov.evaluate(&request_distinct_target(
            ActionType::PostPublish,
            member,
            1,
            i,
        ));
        assert_eq!(r.decision, "allow", "under cap must allow");
    }
    let decision = gov.evaluate(&request_distinct_target(
        ActionType::PostPublish,
        member,
        1,
        2,
    ));
    assert_eq!(decision.decision, "deny");
    assert_eq!(
        decision.guards_failed,
        vec!["daily_cap"],
        "pipeline must short-circuit at guard 1 and report nothing else"
    );
    assert!(decision.guards_passed.is_empty());
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
    //
    // F-AUDIT-47: this test previously looped 5x on `PostPublish`, whose daily
    // cap is 2, so the 3rd call was denied by `daily_cap` and "first 5 must
    // allow" could never hold. `JobApplicationSubmit` is used instead: daily
    // cap 8 (> the 5-per-window pacing limit) and no cooldown rule, so
    // `session_pacing` is the only guard that can fire.
    for i in 0..5 {
        let r = gov.evaluate(&request_distinct_target(
            ActionType::JobApplicationSubmit,
            member,
            1,
            i,
        ));
        assert_eq!(r.decision, "allow", "first 5 must allow");
    }
    let decision = gov.evaluate(&request_distinct_target(
        ActionType::JobApplicationSubmit,
        member,
        1,
        5,
    ));
    assert_eq!(decision.decision, "deny");
    assert_eq!(decision.guards_failed, vec!["session_pacing"]);
    assert_eq!(
        decision.guards_passed.len(),
        7,
        "all seven guards before session_pacing must pass"
    );
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
