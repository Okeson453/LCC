//! Tests for h_c (account health) scoring.

use lcc_compliance::h_c::{compute_health, HealthSignals};

fn make_signals() -> HealthSignals {
    HealthSignals {
        response_rate: 0.6,
        acceptance_rate: 0.4,
        error_rate_24h: 0.05,
        restriction_penalty: 0.1,
        engagement_quality: 0.7,
    }
}

#[test]
fn warmup_band_high() {
    let s = HealthSignals {
        response_rate: 0.95,
        acceptance_rate: 0.85,
        error_rate_24h: 0.0,
        restriction_penalty: 0.0,
        engagement_quality: 0.95,
    };
    let (score, band) = compute_health(&s);
    assert_eq!(band, "warmup");
    assert!(score > 0.85);
}

#[test]
fn standard_band() {
    let s = HealthSignals {
        response_rate: 0.5,
        acceptance_rate: 0.4,
        error_rate_24h: 0.1,
        restriction_penalty: 0.2,
        engagement_quality: 0.5,
    };
    let (score, band) = compute_health(&s);
    assert_eq!(band, "standard");
    assert!(score > 0.65 && score < 0.85);
}

#[test]
fn cold_band() {
    let s = HealthSignals {
        response_rate: 0.1,
        acceptance_rate: 0.05,
        error_rate_24h: 0.5,
        restriction_penalty: 0.5,
        engagement_quality: 0.1,
    };
    let (score, band) = compute_health(&s);
    assert_eq!(band, "cold");
    assert!(score < 0.65);
}

#[test]
fn score_in_unit_interval() {
    let s = make_signals();
    let (score, _) = compute_health(&s);
    assert!(score >= 0.0 && score <= 1.0);
}

#[test]
fn weights_default_values_sum_to_one() {
    use lcc_compliance::h_c::default_weights;
    let w = default_weights();
    let sum: f32 = w.response_rate + w.acceptance_rate + w.error_rate_inverted
        + w.restriction_inverted + w.engagement_quality;
    assert!((sum - 1.0).abs() < 1e-6, "weights must sum to 1.0");
}
