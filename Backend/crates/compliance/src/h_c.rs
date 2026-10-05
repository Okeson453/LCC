//! Account Health Composite (H_c) — Source Technical Design Spec §4.
//!
//! H_c ∈ [0,1] — a single scalar the Compliance Governor consults before
//! permitting elevated-risk action tiers (Source §11 guard 4). Weights are
//! loaded from the active `compliance_config_version` (Source axiom 9) and
//! MUST be calibrated against ≥200 labeled account-days before driving
//! production decisions (Source §29).
//!
//! ```math
//! H_c = w_1 · A_r + w_2 · R_r + w_3 · (1 - Q_u) + w_4 · T_a
//! ```

use serde::{Deserialize, Serialize};

/// Inputs to H_c computation. All values are bounded to [0,1] (clamped at the
/// boundary before computation; see [`h_c`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HCInputs {
    /// A_r — connection-request acceptance rate, trailing 30d
    pub acceptance_rate: f64,
    /// R_r — outreach reply rate, trailing 30d
    pub reply_rate: f64,
    /// Q_u — quota-utilization ratio (actions taken / actions allowed), trailing 7d
    pub quota_utilization: f64,
    /// T_a — account tenure factor, min(1, days_active / 90)
    pub tenure_factor: f64,
}

/// Per-component breakdown (for observability + dashboarding).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HCComponents {
    pub acceptance_rate: f64,
    pub reply_rate: f64,
    pub quota_utilization: f64,
    pub tenure_factor: f64,
    pub weights: [f64; 4],
}

/// Clamp a value to [0,1].
fn clamp01(x: f64) -> f64 {
    if x.is_nan() {
        // Source §32: NaN → H_c=0 (fail-safe strictest caps).
        return 0.0;
    }
    x.clamp(0.0, 1.0)
}

/// Compute H_c.
///
/// Returns 0.0 if any input is NaN (fail-safe). Inputs are clamped to [0,1].
/// Weights are not validated here; the caller should validate at config load.
pub fn h_c(inputs: &HCInputs, weights: &[f64; 4]) -> f64 {
    // Source §32: a NaN signal must produce the strictest caps, not a
    // middling score. Clamping NaN to 0 per-component and then continuing is
    // NOT fail-safe — it silently downgrades one broken signal to "0% on that
    // axis" and still returns a plausible, non-restrictive number.
    if inputs.acceptance_rate.is_nan()
        || inputs.reply_rate.is_nan()
        || inputs.quota_utilization.is_nan()
        || inputs.tenure_factor.is_nan()
    {
        return 0.0;
    }

    let a_r = clamp01(inputs.acceptance_rate);
    let r_r = clamp01(inputs.reply_rate);
    let q_u = clamp01(inputs.quota_utilization);
    let t_a = clamp01(inputs.tenure_factor);

    let w1 = weights[0];
    let w2 = weights[1];
    let w3 = weights[2];
    let w4 = weights[3];

    let result = w1 * a_r + w2 * r_r + w3 * (1.0 - q_u) + w4 * t_a;

    clamp01(result)
}

/// Compute H_c and return the per-component breakdown.
pub fn h_c_with_components(inputs: &HCInputs, weights: &[f64; 4]) -> (f64, HCComponents) {
    let score = h_c(inputs, weights);
    (
        score,
        HCComponents {
            acceptance_rate: clamp01(inputs.acceptance_rate),
            reply_rate: clamp01(inputs.reply_rate),
            quota_utilization: clamp01(inputs.quota_utilization),
            tenure_factor: clamp01(inputs.tenure_factor),
            weights: *weights,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_weights() -> [f64; 4] {
        [0.35, 0.25, 0.20, 0.20]
    }

    #[test]
    fn perfect_health() {
        // A_r=1.0, R_r=1.0, Q_u=0.0 (no quota usage), T_a=1.0
        let inputs = HCInputs {
            acceptance_rate: 1.0,
            reply_rate: 1.0,
            quota_utilization: 0.0,
            tenure_factor: 1.0,
        };
        let score = h_c(&inputs, &default_weights());
        // w1*1.0 + w2*1.0 + w3*1.0 + w4*1.0 = 1.0
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_health() {
        let inputs = HCInputs {
            acceptance_rate: 0.0,
            reply_rate: 0.0,
            quota_utilization: 1.0,
            tenure_factor: 0.0,
        };
        let score = h_c(&inputs, &default_weights());
        // w1*0 + w2*0 + w3*0 + w4*0 = 0
        assert!((score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn warmup_account_below_threshold() {
        // New account: T_a is small, A_r/R_r may be 0
        let inputs = HCInputs {
            acceptance_rate: 0.5,
            reply_rate: 0.3,
            quota_utilization: 0.5,
            tenure_factor: 0.1, // ~9 days active
        };
        let score = h_c(&inputs, &default_weights());
        // Should be in warm-up territory (≤ 0.4)
        assert!(
            score < 0.4,
            "warmup account should score < 0.4: got {score}"
        );
    }

    #[test]
    fn nan_inputs_failsafe_to_zero() {
        let inputs = HCInputs {
            acceptance_rate: f64::NAN,
            reply_rate: 0.5,
            quota_utilization: 0.5,
            tenure_factor: 0.5,
        };
        let score = h_c(&inputs, &default_weights());
        assert_eq!(score, 0.0);
    }

    #[test]
    fn out_of_range_clamped() {
        let inputs = HCInputs {
            acceptance_rate: 2.0, // out of range
            reply_rate: -0.5,     // out of range
            quota_utilization: 0.5,
            tenure_factor: 0.5,
        };
        let score = h_c(&inputs, &default_weights());
        // Should clamp to [0,1] without panicking.
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn components_breakdown_correct() {
        let inputs = HCInputs {
            acceptance_rate: 0.8,
            reply_rate: 0.6,
            quota_utilization: 0.2,
            tenure_factor: 0.9,
        };
        let (score, comps) = h_c_with_components(&inputs, &default_weights());
        let expected = 0.35 * 0.8 + 0.25 * 0.6 + 0.20 * (1.0 - 0.2) + 0.20 * 0.9;
        assert!((score - expected).abs() < 1e-9);
        assert_eq!(comps.acceptance_rate, 0.8);
        assert_eq!(comps.reply_rate, 0.6);
    }
}
