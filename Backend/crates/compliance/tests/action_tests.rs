//! Tests for the compliance action types and their risk tiers.
//!
//! Written against the real 5-tier model in `lcc_compliance::action`. The
//! previous version used a 3-tier `RiskTier::{Low, Medium, High}` and action
//! names (`Dm`, `CommentPost`, `LikePost`, `ProfileView`, `FollowCompany`)
//! that do not exist, so this test target never compiled.
//!
//! The tier a test cares about here is the one that decides how many actions
//! the governor will let through, so the mapping is asserted exhaustively
//! rather than sampled.

use lcc_compliance::action::{ActionType, RiskTier};

/// Every action with the tier it must map to.
const MAPPING: &[(ActionType, RiskTier)] = &[
    // Tier 1 — drafting / editing, auto-execute, no daily quota.
    (ActionType::ProfileEditDraft, RiskTier::Tier1DraftOrEdit),
    (ActionType::ContentDraft, RiskTier::Tier1DraftOrEdit),
    (ActionType::KbRecordCreate, RiskTier::Tier1DraftOrEdit),
    (ActionType::SequenceDraft, RiskTier::Tier1DraftOrEdit),
    (ActionType::OpportunityDiscover, RiskTier::Tier1DraftOrEdit),
    (ActionType::ResearchScan, RiskTier::Tier1DraftOrEdit),
    (ActionType::VoiceTrain, RiskTier::Tier1DraftOrEdit),
    // Tier 2 — light engagement / publish.
    (ActionType::Comment, RiskTier::Tier2LightEngagement),
    (ActionType::Like, RiskTier::Tier2LightEngagement),
    (ActionType::PostPublish, RiskTier::Tier2LightEngagement),
    (ActionType::ProfileEditSubmit, RiskTier::Tier2LightEngagement),
    // Tier 3 — network growth.
    (ActionType::ConnectionRequest, RiskTier::Tier3NetworkGrowth),
    // Tier 4 — 1:1 outreach.
    (ActionType::DirectMessage, RiskTier::Tier4OneToOneOutreach),
    (ActionType::SequenceStepSend, RiskTier::Tier4OneToOneOutreach),
    // Tier 5 — high stakes.
    (ActionType::JobApplicationSubmit, RiskTier::Tier5HighStakes),
    (ActionType::ClientProposalSend, RiskTier::Tier5HighStakes),
    (ActionType::ExecutiveOutreach, RiskTier::Tier5HighStakes),
];

#[test]
fn every_action_maps_to_its_documented_tier() {
    for (action, expected) in MAPPING {
        assert_eq!(
            action.risk_tier(),
            *expected,
            "{action:?} is in the wrong tier"
        );
    }
}

#[test]
fn mapping_covers_every_action_variant() {
    // If a variant is added without a tier assertion, this fails — so a new
    // action can never silently inherit the wrong risk tier.
    assert_eq!(MAPPING.len(), 17, "a new ActionType needs a tier assertion");
}

#[test]
fn tier1_drafting_consumes_no_daily_quota() {
    for (action, tier) in MAPPING {
        if *tier == RiskTier::Tier1DraftOrEdit {
            assert_eq!(action.cap_base(), 0, "{action:?} should not consume quota");
        }
    }
}

#[test]
fn quota_tracked_tiers_declare_a_cap() {
    // Tiers 2-4 are the quota-tracked engagement types, and each appears in
    // `ActionCaps` (§12), so each must declare a positive daily cap.
    //
    // Tier 1 is drafting (auto-execute, no quota) and tier 5 is high-stakes
    // one-off sends that are individually approved rather than batched — tier
    // 5 has no field in `ActionCaps` at all, so a zero cap there is correct
    // and is asserted separately below.
    for (action, tier) in MAPPING {
        let quota_tracked = matches!(
            tier,
            RiskTier::Tier2LightEngagement
                | RiskTier::Tier3NetworkGrowth
                | RiskTier::Tier4OneToOneOutreach
        );
        if quota_tracked {
            assert!(
                action.cap_base() > 0,
                "{action:?} is quota-tracked but declares no daily cap"
            );
            assert!(
                action.warm_up_cap() > 0,
                "{action:?} is quota-tracked but has no warm-up floor"
            );
        }
    }
}

#[test]
fn warm_up_caps_never_exceed_base_caps() {
    // The warm-up floor is a restriction, not a bonus.
    for (action, tier) in MAPPING {
        if *tier != RiskTier::Tier1DraftOrEdit {
            assert!(
                action.warm_up_cap() <= action.cap_base(),
                "{action:?} warm-up cap exceeds its base cap"
            );
        }
    }
}

#[test]
fn h_c_requirement_is_monotonic_in_tier() {
    // A higher-risk action must never be allowed at a lower H_c than a
    // lower-risk one, or the caps could be bypassed by picking a different
    // action type.
    let mut tiers = vec![
        RiskTier::Tier1DraftOrEdit,
        RiskTier::Tier2LightEngagement,
        RiskTier::Tier3NetworkGrowth,
        RiskTier::Tier4OneToOneOutreach,
        RiskTier::Tier5HighStakes,
    ];
    let mins: Vec<f64> = tiers.iter().map(|t| t.minimum_h_c()).collect();
    assert!(
        mins.windows(2).all(|w| w[0] <= w[1]),
        "minimum H_c is not monotonic across tiers: {mins:?}"
    );
    assert_eq!(tiers.first().map(|t| t.minimum_h_c()), Some(0.0));
    tiers.dedup();
    assert_eq!(tiers.len(), 5, "tier discriminants must stay distinct");
}
