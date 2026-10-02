//! Tests for ρ (rho) scoring.
//!
//! These assert the five behaviours the scorer is required to have. They are
//! written against the real `lcc_compliance::rho` surface
//! (`ReplyProbabilityFeatures` / `rho_with_mode` / `RhoResult`); the previous
//! version imported a `ContactFeature` / `score_contact` API that does not
//! exist, so this test target never compiled.

use lcc_compliance::rho::{rho_with_mode, ReplyProbabilityFeatures, RhoMode};

/// A warm, mutual, VIP contact — the high-signal baseline.
fn make_vip_contact() -> ReplyProbabilityFeatures {
    ReplyProbabilityFeatures {
        member_id: "m1".into(),
        contact_id: "c1".into(),
        mutual_count: 25,
        personalization_score: 0.8,
        prior_interaction_flag: true,
        contact_tier: "VIP".into(),
        last_interaction_age_days: 5,
        tag_match_flags: vec!["ai".into(), "dev".into()],
    }
}

/// Betas are only consulted once the account is calibrated (≥200 labeled
/// sends); below that the rule-based path is used and these are ignored.
const BETAS: [f64; 4] = [0.1, 0.02, 1.5, 0.8];

#[test]
fn cold_start_uses_rule_based() {
    let contact = make_vip_contact();
    let result = rho_with_mode(&contact, Some(&BETAS), 50, None);
    assert_eq!(result.mode, RhoMode::RuleBasedFallback);
    assert!(result.rho > 0.0);
    // Axiom: an uncalibrated account must not present a model score.
    assert!(result.model_version.is_none());
}

#[test]
fn vip_scores_higher_than_non_vip() {
    let mut vip = make_vip_contact();
    vip.contact_tier = "VIP".into();
    let mut plain = make_vip_contact();
    plain.contact_tier = "peer".into();

    let s_vip = rho_with_mode(&vip, Some(&BETAS), 50, None);
    let s_plain = rho_with_mode(&plain, Some(&BETAS), 50, None);
    assert!(s_vip.rho > s_plain.rho);
}

#[test]
fn fresh_contact_scores_higher_than_stale() {
    let mut fresh = make_vip_contact();
    fresh.last_interaction_age_days = 1;
    let mut stale = make_vip_contact();
    stale.last_interaction_age_days = 120;

    let s_fresh = rho_with_mode(&fresh, Some(&BETAS), 50, None);
    let s_stale = rho_with_mode(&stale, Some(&BETAS), 50, None);
    assert!(s_fresh.rho >= s_stale.rho);
}

#[test]
fn warm_start_uses_the_calibrated_model() {
    let contact = make_vip_contact();
    let result = rho_with_mode(&contact, Some(&BETAS), 250, Some("rho-2026.01".into()));
    assert_eq!(result.mode, RhoMode::ModelInference);
    assert_eq!(result.labeled_send_count, 250);
    assert_eq!(result.model_version.as_deref(), Some("rho-2026.01"));
}

#[test]
fn score_in_unit_interval() {
    for count in [0_u32, 50, 199, 200, 1000] {
        let contact = make_vip_contact();
        let r = rho_with_mode(&contact, Some(&BETAS), count, None);
        assert!(
            (0.0..=1.0).contains(&r.rho),
            "rho {} out of [0,1] at labeled_send_count={count}",
            r.rho
        );
    }
}
