//! Tests for compliance config validation.
//!
//! The YAML in these tests uses the flat `ComplianceConfig` schema — the same
//! shape as `config/compliance/*.yaml`. The previous version used a nested
//! `h_c.weights.{...}` shape the loader cannot read, so every case here
//! "passed" by failing to deserialise rather than by hitting the invariant it
//! claimed to test.

use lcc_compliance::config::ComplianceConfig;

const VALID: &str = r#"
version: ccfg-test
h_c_weights: [0.35, 0.25, 0.20, 0.20]
h_c_warmup_threshold: 0.40
h_c_standard_threshold: 0.70
reserve_fraction: 0.10
phi_weights: [0.35, 0.30, 0.20, 0.15]
phi_qualification_threshold: 0.65
ab_d_multiplier_floor: 0.5
"#;

/// `VALID` with one line swapped, so each case differs from the known-good
/// config in exactly one way.
fn config_with(from: &str, to: &str) -> String {
    assert!(VALID.contains(from), "`{from}` must be a line of VALID");
    VALID.replacen(from, to, 1)
}

#[test]
fn valid_config_parses() {
    let cfg = ComplianceConfig::from_yaml_str(VALID).expect("valid config should parse");
    assert_eq!(cfg.version, "ccfg-test");
    assert_eq!(cfg.h_c_weights, [0.35, 0.25, 0.20, 0.20]);
    assert_eq!(cfg.reserve_fraction, 0.10);
    // Fields with a serde default are populated even though the YAML omits them.
    assert_eq!(cfg.caps.connection_request, 18);
    assert_eq!(cfg.duplicate_target_cooldown_days, 30);
}

#[test]
fn weights_summing_to_one_are_accepted() {
    let cfg = ComplianceConfig::from_yaml_str(VALID).expect("must load");
    assert_eq!(cfg.h_c_weights.iter().sum::<f64>(), 1.0);
}

#[test]
fn weights_not_summing_to_one_are_rejected() {
    let yaml = config_with("h_c_weights: [0.35, 0.25, 0.20, 0.20]", "h_c_weights: [0.35, 0.25, 0.20, 0.10]");
    let result = ComplianceConfig::from_yaml_str(&yaml);
    assert!(result.is_err(), "weights summing to 0.90 must be rejected");
}

#[test]
fn warmup_must_be_below_standard() {
    // h_c_warmup_threshold is the H_c *below* which an account is still
    // warming up, so it must sit below the standard threshold.
    let yaml = config_with("h_c_warmup_threshold: 0.40", "h_c_warmup_threshold: 0.70");
    let result = ComplianceConfig::from_yaml_str(&yaml);
    assert!(result.is_err(), "warmup == standard must be rejected");
}

#[test]
fn reserve_fraction_too_high() {
    let yaml = config_with("reserve_fraction: 0.10", "reserve_fraction: 0.50");
    let result = ComplianceConfig::from_yaml_str(&yaml);
    assert!(result.is_err(), "reserve_fraction > 0.30 must be rejected");
}

#[test]
fn ab_d_multiplier_floor_out_of_range() {
    // The multiplier must never be allowed to reach zero, or the adaptive
    // multiplier would silently stop damping anything.
    let yaml = config_with("ab_d_multiplier_floor: 0.5", "ab_d_multiplier_floor: 1.5");
    let result = ComplianceConfig::from_yaml_str(&yaml);
    assert!(result.is_err(), "a floor of 1.5 (above 1) must be rejected");
}

/// Every configuration shipped in the repo must load.
///
/// This is the invariant that was silently broken: `config/compliance/*.yaml`
/// used a nested schema the loader could not read, so the Governor could not
/// start against any of them and nothing asserted that the files were loadable.
#[test]
fn every_shipped_config_loads() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/compliance");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).expect("config/compliance is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let yaml = std::fs::read_to_string(&path).expect("read config");
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let cfg = ComplianceConfig::from_yaml_str(&yaml)
            .unwrap_or_else(|e| panic!("{name} failed to load: {e}"));
        assert_eq!(cfg.version, name.trim_end_matches(".yaml"));
        cfg.validate().unwrap_or_else(|e| panic!("{name} is invalid: {e}"));
        checked += 1;
    }
    assert!(checked > 0, "no compliance configs found in {dir:?}");
}

/// The configs are ordered from permissive to restrictive; if that progression
/// is lost, a "tighter" release could ship looser caps than its parent.
#[test]
fn configs_get_stricter_across_releases() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/compliance");
    let load = |name: &str| -> ComplianceConfig {
        let yaml = std::fs::read_to_string(dir.join(name)).expect("read config");
        ComplianceConfig::from_yaml_str(&yaml).unwrap_or_else(|e| panic!("{name}: {e}"))
    };

    let rc1 = load("ccfg-2025-01-01-rc1.yaml");
    let rc2 = load("ccfg-2025-01-15-rc2.yaml");
    let canary = load("ccfg-2025-02-01-canary.yaml");

    assert!(
        rc2.caps.direct_message < rc1.caps.direct_message,
        "rc2 must tighten the DM cap relative to rc1"
    );
    assert!(
        canary.caps.direct_message < rc2.caps.direct_message,
        "the canary must tighten the DM cap relative to rc2"
    );
    assert!(rc2.reserve_fraction > rc1.reserve_fraction);
    assert!(canary.reserve_fraction > rc2.reserve_fraction);
    assert!(rc2.h_c_warmup_threshold > rc1.h_c_warmup_threshold);
    assert!(canary.h_c_warmup_threshold > rc2.h_c_warmup_threshold);
    assert!(canary.duplicate_target_cooldown_days > rc1.duplicate_target_cooldown_days);
}
