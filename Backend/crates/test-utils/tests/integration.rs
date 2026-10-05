//! Tests for the shared test harnesses.
//!
//! The previous version of this file was a placeholder that asserted `true`.
//! It also imported the crate as `test_utils`, a name that does not exist
//! (the package is `lcc-test-utils`), so the target never compiled at all.
//!
//! These cover the helpers that need no external infrastructure. Anything
//! needing a live Postgres or Redis — `connect_test_postgres`,
//! `test_redis_pool`, `apply_migrations` — is exercised by the service test
//! suites that actually have a database, not from here.

use lcc_test_utils::governor::TestScenario;
use lcc_test_utils::redis::unique_prefix;
use uuid::Uuid;

#[test]
fn unique_prefix_is_usable_as_a_redis_key_prefix() {
    let a = unique_prefix();
    let b = unique_prefix();

    assert!(a.starts_with("test:"), "prefix must be namespaced: {a}");
    assert!(a.ends_with(':'), "prefix must end with a separator: {a}");
    assert_ne!(a, b, "two prefixes must not collide");
}

#[test]
fn happy_path_scenario_scores_above_the_fallback_scenarios() {
    let member = Uuid::now_v7();
    let happy = TestScenario::happy_path(member).h_c();
    let low = TestScenario::low_h_c(member).h_c();

    assert!(
        (0.0..=1.0).contains(&happy),
        "H_c must stay in [0, 1], got {happy}"
    );
    assert!(
        happy > low,
        "a new account should score below a healthy one"
    );
    // `low_grounding` scores identically to `happy_path`: it clears
    // `kb_refs`, which guard 5 reads, and H_c is computed from the four
    // `h_c_inputs` only. It is not a weaker-signal fixture, and treating it as
    // one would hide a regression that let grounding influence H_c.
    let low_grounding = TestScenario::low_grounding(member).h_c();
    assert_eq!(low_grounding, happy, "grounding must not move H_c");
}

#[test]
fn low_h_c_scenario_is_the_weakest_signal() {
    // The four inputs of `low_h_c` are all at or near the bottom of their
    // range, so the weighted score cannot exceed the happy path's. This is the
    // property the helper exists to guarantee for guard 4.
    let member = Uuid::now_v7();
    let low = TestScenario::low_h_c(member);
    let score = low.h_c();

    assert!((0.0..=1.0).contains(&score), "H_c out of range: {score}");
    assert!(
        score < 0.5,
        "a near-zero-input scenario should be low: {score}"
    );
    assert_eq!(low.action.risk_tier, 3, "guard 4 applies from tier 3 up");
}

#[test]
fn grounding_scenario_clears_its_refs() {
    // `low_grounding` is the "should fail guard 5" fixture, which only holds
    // if there is genuinely nothing to ground the action in.
    let s = TestScenario::low_grounding(Uuid::now_v7());
    assert!(s.action.kb_refs.is_empty(), "expected no KB refs");
    assert_eq!(s.action.action_type, "post_publish");
    assert!(!s.action.requires_approval);
}
