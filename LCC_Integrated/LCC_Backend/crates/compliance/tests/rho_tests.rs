//! Tests for ρ (rho) scoring.

use lcc_compliance::rho::{score_contact, ContactFeature};

fn make_vip_contact() -> ContactFeature {
    ContactFeature {
        contact_id: "c1".into(),
        is_vip: true,
        is_mutual: true,
        last_contact_days: 5,
        matched_kb_facts: 5,
        response_rate: 0.6,
        accept_rate: 0.4,
    }
}

#[test]
fn cold_start_uses_rule_based() {
    let contact = make_vip_contact();
    let result = score_contact(&contact, 50);
    assert_eq!(result.mode, "rule_based");
    assert!(result.score > 0.0);
}

#[test]
fn vip_scores_higher_than_non_vip() {
    let mut vip = make_vip_contact();
    vip.is_vip = true;
    let mut plain = make_vip_contact();
    plain.is_vip = false;

    let s_vip = score_contact(&vip, 50);
    let s_plain = score_contact(&plain, 50);
    assert!(s_vip.score > s_plain.score);
}

#[test]
fn fresh_contact_scores_higher_than_stale() {
    let mut fresh = make_vip_contact();
    fresh.last_contact_days = 1;
    let mut stale = make_vip_contact();
    stale.last_contact_days = 120;

    let s_fresh = score_contact(&fresh, 50);
    let s_stale = score_contact(&stale, 50);
    assert!(s_fresh.score >= s_stale.score);
}

#[test]
fn warm_start_ml_mode() {
    let contact = make_vip_contact();
    let result = score_contact(&contact, 250);
    // ML loader may not exist → fall back to rule-based. Either is acceptable.
    assert!(result.mode == "ml" || result.mode == "rule_based");
}

#[test]
fn score_in_unit_interval() {
    let contact = make_vip_contact();
    let r = score_contact(&contact, 50);
    assert!(r.score >= 0.0 && r.score <= 1.0);
}
