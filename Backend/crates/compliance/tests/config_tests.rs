//! Tests for compliance config validation.

use lcc_compliance::config::ComplianceConfig;

#[test]
fn valid_config_parses() {
    let yaml = r#"
version: ccfg-test-1
h_c:
  weights:
    response_rate: 0.30
    acceptance_rate: 0.20
    error_rate_inverted: 0.20
    restriction_inverted: 0.15
    engagement_quality: 0.15
  warmup_threshold: 0.85
  standard_threshold: 0.65
  reserve_fraction: 0.10
ab_d:
  multiplier_floor: 0.5
  multiplier_ceiling: 2.0
rho:
  labeled_send_threshold: 200
  vip_boost: 1.5
phi:
  qualified_threshold: 0.65
"#;
    let cfg = ComplianceConfig::from_yaml_str(yaml).expect("valid config should parse");
    assert_eq!(cfg.version, "ccfg-test-1");
}

#[test]
fn weights_must_sum_to_one() {
    let yaml = r#"
version: ccfg-test
h_c:
  weights:
    response_rate: 0.30
    acceptance_rate: 0.20
    error_rate_inverted: 0.20
    restriction_inverted: 0.15
    engagement_quality: 0.05
  warmup_threshold: 0.85
  standard_threshold: 0.65
  reserve_fraction: 0.10
ab_d:
  multiplier_floor: 0.5
  multiplier_ceiling: 2.0
rho:
  labeled_send_threshold: 200
  vip_boost: 1.5
phi:
  qualified_threshold: 0.65
"#;
    let result = ComplianceConfig::from_yaml_str(yaml);
    assert!(result.is_err(), "weights summing to 0.90 must be rejected");
}

#[test]
fn warmup_must_exceed_standard() {
    let yaml = r#"
version: ccfg-test
h_c:
  weights:
    response_rate: 1.0
  warmup_threshold: 0.65
  standard_threshold: 0.85
  reserve_fraction: 0.10
ab_d:
  multiplier_floor: 0.5
  multiplier_ceiling: 2.0
rho:
  labeled_send_threshold: 200
  vip_boost: 1.5
phi:
  qualified_threshold: 0.65
"#;
    let result = ComplianceConfig::from_yaml_str(yaml);
    assert!(result.is_err(), "warmup < standard must be rejected");
}

#[test]
fn reserve_fraction_too_high() {
    let yaml = r#"
version: ccfg-test
h_c:
  weights:
    response_rate: 1.0
  warmup_threshold: 0.85
  standard_threshold: 0.65
  reserve_fraction: 0.50
ab_d:
  multiplier_floor: 0.5
  multiplier_ceiling: 2.0
rho:
  labeled_send_threshold: 200
  vip_boost: 1.5
phi:
  qualified_threshold: 0.65
"#;
    let result = ComplianceConfig::from_yaml_str(yaml);
    assert!(result.is_err(), "reserve_fraction > 0.30 must be rejected");
}
