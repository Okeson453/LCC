//! Tests for H_c (account health) scoring.
//!
//! Written against the real `lcc_compliance::h_c` surface
//! (`HCInputs` / `h_c` / `h_c_with_components`). The previous version
//! imported a `HealthSignals` / `compute_health` API with a warmup/standard/
//! cold "band" that does not exist, so this test target never compiled.
//!
//! H_c is the governor's health scalar: it lowers the outbound caps as an
//! account degrades, so the properties worth pinning are monotonicity, the
//! unit interval, weight normalisation, and the fail-safe on bad input.

use lcc_compliance::h_c::{h_c, h_c_with_components, HCInputs};

/// The documented default weights (ComplianceConfig::default). They sum to 1.0,
/// which `ComplianceConfig::validate` enforces.
const DEFAULT_WEIGHTS: [f64; 4] = [0.35, 0.25, 0.20, 0.20];

/// A healthy, established account.
fn healthy() -> HCInputs {
    HCInputs {
        acceptance_rate: 0.85,
        reply_rate: 0.80,
        quota_utilization: 0.10,
        tenure_factor: 1.0,
    }
}

/// A new account with poor signal.
fn unhealthy() -> HCInputs {
    HCInputs {
        acceptance_rate: 0.10,
        reply_rate: 0.05,
        quota_utilization: 0.95,
        tenure_factor: 0.1,
    }
}

#[test]
fn healthy_account_scores_high() {
    let score = h_c(&healthy(), &DEFAULT_WEIGHTS);
    assert!(score > 0.8, "healthy account scored {score}");
}

#[test]
fn unhealthy_account_scores_low() {
    let score = h_c(&unhealthy(), &DEFAULT_WEIGHTS);
    assert!(score < 0.4, "unhealthy account scored {score}");
}

#[test]
fn health_is_monotonic_in_acceptance_and_reply_rate() {
    let base = healthy();
    let mut worse = base;
    worse.acceptance_rate = 0.2;
    worse.reply_rate = 0.2;
    assert!(h_c(&worse, &DEFAULT_WEIGHTS) < h_c(&base, &DEFAULT_WEIGHTS));
}

#[test]
fn heavier_quota_utilization_lowers_health() {
    let base = healthy();
    let mut hammered = base;
    hammered.quota_utilization = 1.0;
    assert!(h_c(&hammered, &DEFAULT_WEIGHTS) < h_c(&base, &DEFAULT_WEIGHTS));
}

#[test]
fn score_in_unit_interval() {
    for inputs in [
        healthy(),
        unhealthy(),
        HCInputs {
            acceptance_rate: 0.5,
            reply_rate: 0.5,
            quota_utilization: 0.5,
            tenure_factor: 0.5,
        },
    ] {
        let s = h_c(&inputs, &DEFAULT_WEIGHTS);
        assert!((0.0..=1.0).contains(&s), "score {s} out of [0,1]");
    }
}

#[test]
fn out_of_range_inputs_are_clamped_not_propagated() {
    let wild = HCInputs {
        acceptance_rate: 5.0,
        reply_rate: -3.0,
        quota_utilization: 9.0,
        tenure_factor: 2.0,
    };
    let s = h_c(&wild, &DEFAULT_WEIGHTS);
    assert!(
        (0.0..=1.0).contains(&s),
        "score {s} out of [0,1] after clamp"
    );
}

#[test]
fn nan_input_fails_safe_to_zero() {
    // Source §32: a NaN signal must produce the strictest caps, not a panic
    // and not a silently healthy score.
    let broken = HCInputs {
        acceptance_rate: f64::NAN,
        reply_rate: 0.5,
        quota_utilization: 0.5,
        tenure_factor: 0.5,
    };
    assert_eq!(h_c(&broken, &DEFAULT_WEIGHTS), 0.0);
}

#[test]
fn components_echo_the_inputs() {
    let inputs = healthy();
    let (score, components) = h_c_with_components(&inputs, &DEFAULT_WEIGHTS);
    assert_eq!(components.acceptance_rate, inputs.acceptance_rate);
    assert_eq!(components.reply_rate, inputs.reply_rate);
    assert_eq!(components.quota_utilization, inputs.quota_utilization);
    assert_eq!(components.tenure_factor, inputs.tenure_factor);
    assert_eq!(components.weights, DEFAULT_WEIGHTS);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn default_weights_sum_to_one() {
    let sum: f64 = DEFAULT_WEIGHTS.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-9,
        "weights must sum to 1.0, got {sum}"
    );
}
