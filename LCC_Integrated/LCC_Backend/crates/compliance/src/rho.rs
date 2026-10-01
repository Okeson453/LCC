//! Reply-Probability Estimate (ρ) — Source Technical Design Spec §5, §29.
//!
//! IMPORTANT: ρ coefficients (β) are NOT shipped with production defaults.
//! Until ≥200 labeled sends exist for a member, the system MUST fall back to
//! rule-based priority order (VIP tag > recency > mutual count), stated
//! explicitly rather than substituting an uncalibrated model silently.
//!
//! This module provides:
//! - [`rho`] — model inference when calibrated.
//! - [`RhoMode::RuleBasedFallback`] — rule-based priority order for under-calibrated accounts.
//! - [`RhoResult`] — discriminated result with reasoning.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplyProbabilityFeatures {
    pub member_id: String,
    pub contact_id: String,
    pub mutual_count: u32,
    pub personalization_score: f64,
    pub prior_interaction_flag: bool,
    pub contact_tier: String,           // "VIP"|"standard"|"peer"
    pub last_interaction_age_days: u32,
    pub tag_match_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RhoMode {
    /// Calibrated model inference — used when labeled_send_count ≥ 200.
    ModelInference,
    /// Rule-based fallback — used when labeled_send_count < 200.
    RuleBasedFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RhoResult {
    pub rho: f64,                          // [0,1]
    pub mode: RhoMode,
    pub labeled_send_count: u32,
    pub model_version: Option<String>,
    pub reason: String,
}

/// Logistic function σ(x) = 1 / (1 + e^{-x}).
pub fn logistic(x: f64) -> f64 {
    if x >= 0.0 {
        let e = (-x).exp();
        1.0 / (1.0 + e)
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}

/// Compute ρ via the calibrated logistic model.
///
/// `betas` MUST come from a fitted model trained on the account's own labeled
/// send/reply data (Source §29). The caller is responsible for verifying the
/// calibration has ≥200 labeled sends before calling this; use [`rho_with_mode`]
/// for the explicit rule-based fallback decision.
///
/// ```math
/// ρ = σ(β_0 + β_1·mutual_count + β_2·personalization_score + β_3·prior_interaction_flag)
/// ```
pub fn rho(features: &ReplyProbabilityFeatures, betas: &[f64; 4]) -> f64 {
    let prior = if features.prior_interaction_flag { 1.0 } else { 0.0 };
    let x = betas[0]
        + betas[1] * features.mutual_count as f64
        + betas[2] * features.personalization_score
        + betas[3] * prior;
    logistic(x)
}

/// Compute ρ with the explicit fallback decision.
///
/// If `labeled_send_count < 200`, returns the rule-based priority score (VIP >
/// recency > mutual count) instead of the model — Source §5 / §29.
pub fn rho_with_mode(
    features: &ReplyProbabilityFeatures,
    betas: Option<&[f64; 4]>,
    labeled_send_count: u32,
    model_version: Option<String>,
) -> RhoResult {
    if labeled_send_count < 200 {
        let rule_score = rule_based_score(features);
        RhoResult {
            rho: rule_score,
            mode: RhoMode::RuleBasedFallback,
            labeled_send_count,
            model_version,
            reason: format!(
                "labeled_send_count={} < 200; rule-based fallback in effect",
                labeled_send_count
            ),
        }
    } else {
        let betas = betas.expect("calibrated model required when labeled_send_count ≥ 200");
        let score = rho(features, betas);
        RhoResult {
            rho: score,
            mode: RhoMode::ModelInference,
            labeled_send_count,
            model_version,
            reason: "calibrated model inference".to_string(),
        }
    }
}

/// Rule-based priority score (VIP > recency > mutual count).
///
/// Used as fallback when sample size < 200 labeled sends. Output is a heuristic
/// score in [0,1] used only to ORDER the engagement queue — not a calibrated
/// probability.
fn rule_based_score(features: &ReplyProbabilityFeatures) -> f64 {
    let tier_score = match features.contact_tier.as_str() {
        "VIP" => 1.0,
        "standard" => 0.5,
        "peer" => 0.25,
        _ => 0.1,
    };

    // Recency: 1.0 if interacted within 7 days, 0.5 within 30 days, else proportional.
    let recency_score = if features.last_interaction_age_days <= 7 {
        1.0
    } else if features.last_interaction_age_days <= 30 {
        0.5
    } else if features.last_interaction_age_days <= 90 {
        0.25
    } else {
        0.0
    };

    // Mutual count: capped at 50; saturates around 25 mutuals.
    let mutual_score = (features.mutual_count as f64 / 25.0).min(1.0);

    // Weighted: tier 50%, recency 30%, mutual 20%.
    0.50 * tier_score + 0.30 * recency_score + 0.20 * mutual_score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vip() -> ReplyProbabilityFeatures {
        ReplyProbabilityFeatures {
            member_id: "m_001".into(),
            contact_id: "c_001".into(),
            mutual_count: 10,
            personalization_score: 0.8,
            prior_interaction_flag: true,
            contact_tier: "VIP".into(),
            last_interaction_age_days: 3,
            tag_match_flags: vec!["CTO".into()],
        }
    }

    #[test]
    fn logistic_zero() {
        assert!((logistic(0.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn logistic_positive_high() {
        assert!(logistic(10.0) > 0.99);
    }

    #[test]
    fn logistic_negative_high() {
        assert!(logistic(-10.0) < 0.01);
    }

    #[test]
    fn rho_model_inference_returns_value_in_unit_interval() {
        let features = vip();
        let betas = [-2.0, 0.05, 1.5, 0.8];
        let score = rho(&features, &betas);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn under_threshold_uses_rule_based() {
        let features = vip();
        let result = rho_with_mode(&features, None, 150, None);
        assert!(matches!(result.mode, RhoMode::RuleBasedFallback));
        assert_eq!(result.labeled_send_count, 150);
        assert!(result.reason.contains("< 200"));
    }

    #[test]
    fn at_threshold_uses_model() {
        let features = vip();
        let betas = [-2.0, 0.05, 1.5, 0.8];
        let result = rho_with_mode(&features, Some(&betas), 200, Some("rho-2026-04-12-r1".into()));
        assert!(matches!(result.mode, RhoMode::ModelInference));
        assert_eq!(result.model_version, Some("rho-2026-04-12-r1".into()));
    }

    #[test]
    fn vip_ranks_above_peer() {
        let mut peer = vip();
        peer.contact_tier = "peer".into();

        let vip_result = rho_with_mode(&vip(), None, 50, None);
        let peer_result = rho_with_mode(&peer, None, 50, None);
        assert!(vip_result.rho > peer_result.rho);
    }

    #[test]
    fn recent_interaction_ranks_above_old() {
        let mut old = vip();
        old.last_interaction_age_days = 180;
        let recent = rho_with_mode(&vip(), None, 50, None);
        let old_result = rho_with_mode(&old, None, 50, None);
        assert!(recent.rho > old_result.rho);
    }
}
