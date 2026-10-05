//! `ComplianceConfig` — versioned, two-reviewer-signed configuration.
//!
//! Loaded at service start from `config/compliance/ccfg-*.yaml`. Changes
//! require a separate CI pipeline (`infra/ci/compliance-config-ci.yaml`) with
//! two-reviewer sign-off (Non-Negotiable §4).
//!
//! The struct mirrors `proto/lcc/v1/compliance/admin.proto::ComplianceConfigPayload`.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub const DEFAULT_VERSION: &str = "ccfg-2026-04-12-r3";

/// Per-action daily caps (Scenario B standard, warm-up floor).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionCaps {
    pub connection_request: u32,
    pub direct_message: u32,
    pub comment: u32,
    pub like: u32,
    pub profile_edit_submission: u32,
    pub post_publish: u32,
}

impl Default for ActionCaps {
    fn default() -> Self {
        // Scenario B — Source §12
        Self {
            connection_request: 18,
            direct_message: 25,
            comment: 15,
            like: 40,
            profile_edit_submission: 3,
            post_publish: 3,
        }
    }
}

impl ActionCaps {
    /// Warm-up floor — Scenario C (Source §12).
    pub fn warm_up_floor() -> Self {
        Self {
            connection_request: 5,
            direct_message: 6,
            comment: 4,
            like: 10,
            profile_edit_submission: 1,
            post_publish: 1,
        }
    }
}

/// Minimum inter-action spacing, in seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spacing {
    pub connection_request: u64,
    pub direct_message: u64,
    pub comment: u64,
    pub like: u64,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            connection_request: 90,
            direct_message: 60,
            comment: 45,
            like: 15,
        }
    }
}

/// RiskTier minimum H_c thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskTierThresholds {
    pub tier_1: f64,
    pub tier_2: f64,
    pub tier_3: f64,
    pub tier_4: f64,
    pub tier_5: f64,
}

impl Default for RiskTierThresholds {
    fn default() -> Self {
        Self {
            tier_1: 0.0,
            tier_2: 0.3,
            tier_3: 0.4,
            tier_4: 0.5,
            tier_5: 0.5,
        }
    }
}

/// The full compliance config payload — versioned, loaded at start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub version: String,

    /// H_c weights — must sum to 1.0; range (0,1].
    pub h_c_weights: [f64; 4],

    /// Standard caps (Scenario B by default).
    #[serde(default)]
    pub caps: ActionCaps,

    /// Warm-up floor caps (Scenario C).
    #[serde(default = "ActionCaps::warm_up_floor")]
    pub warm_up_floor_caps: ActionCaps,

    /// Minimum inter-action spacing.
    #[serde(default)]
    pub min_spacing_seconds: Spacing,

    /// φ weights (job-track); client-track substitutes trigger-recency for comp.
    pub phi_weights: [f64; 4],

    /// Reserve fraction (default 0.15).
    #[serde(default = "default_reserve_fraction")]
    pub reserve_fraction: f64,

    /// RiskTier minimum H_c.
    #[serde(default)]
    pub risk_tier_min_h_c: RiskTierThresholds,

    /// Duplicate-target cooldown (days). Default 30.
    #[serde(default = "default_dup_target_cooldown_days")]
    pub duplicate_target_cooldown_days: i64,

    /// Session pacing jitter minimum (seconds). Default 8.
    #[serde(default = "default_session_jitter_seconds")]
    pub session_jitter_min_seconds: u64,

    /// φ qualification threshold (default 0.55).
    #[serde(default = "default_phi_qualification_threshold")]
    pub phi_qualification_threshold: f64,

    /// Permit token TTL (seconds). Default 60.
    #[serde(default = "default_permit_token_ttl_seconds")]
    pub permit_token_ttl_seconds: u64,

    /// H_c thresholds for warm-up vs standard interpolation.
    #[serde(default = "default_warmup_threshold")]
    pub h_c_warmup_threshold: f64,
    #[serde(default = "default_standard_threshold")]
    pub h_c_standard_threshold: f64,

    /// AB_d multiplier floor (default 0.3 — never zero).
    #[serde(default = "default_ab_d_multiplier_floor")]
    pub ab_d_multiplier_floor: f64,

    /// API_SUPPORTED_SET — Track A routing list (JSON-encoded for forward compat).
    #[serde(default = "default_api_supported_set")]
    pub api_supported_set_json: String,
}

fn default_reserve_fraction() -> f64 {
    0.15
}
fn default_dup_target_cooldown_days() -> i64 {
    30
}
fn default_session_jitter_seconds() -> u64 {
    8
}
fn default_phi_qualification_threshold() -> f64 {
    0.55
}
fn default_permit_token_ttl_seconds() -> u64 {
    60
}
fn default_warmup_threshold() -> f64 {
    0.4
}
fn default_standard_threshold() -> f64 {
    0.7
}
fn default_ab_d_multiplier_floor() -> f64 {
    0.3
}
fn default_api_supported_set() -> String {
    r#"["organization_page_post_publish","oauth_profile_read","jobs_data_read"]"#.to_string()
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            version: DEFAULT_VERSION.to_string(),
            // Initial default w=(0.35, 0.25, 0.20, 0.20) — Source §4.
            // Note: must be calibrated against ≥200 labeled account-days before
            // driving auto-tier decisions; until then, treat as initial default.
            h_c_weights: [0.35, 0.25, 0.20, 0.20],
            caps: ActionCaps::default(),
            warm_up_floor_caps: ActionCaps::warm_up_floor(),
            min_spacing_seconds: Spacing::default(),
            // φ weights: skill, seniority, geo, comp — Source §6.3
            phi_weights: [0.40, 0.25, 0.20, 0.15],
            reserve_fraction: default_reserve_fraction(),
            risk_tier_min_h_c: RiskTierThresholds::default(),
            duplicate_target_cooldown_days: default_dup_target_cooldown_days(),
            session_jitter_min_seconds: default_session_jitter_seconds(),
            phi_qualification_threshold: default_phi_qualification_threshold(),
            permit_token_ttl_seconds: default_permit_token_ttl_seconds(),
            h_c_warmup_threshold: default_warmup_threshold(),
            h_c_standard_threshold: default_standard_threshold(),
            ab_d_multiplier_floor: default_ab_d_multiplier_floor(),
            api_supported_set_json: default_api_supported_set(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ComplianceConfigError {
    #[error("io error reading config: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid: weights must sum to 1.0 within 1e-6; got {0}")]
    InvalidWeights(f64),
    #[error("invalid: weight[{0}] = {1} not in (0,1]")]
    InvalidWeightComponent(usize, f64),
    #[error("invalid: H_c warmup_threshold ({0}) must be < standard_threshold ({1})")]
    InvalidHCThresholds(f64, f64),
    #[error("invalid: reserve_fraction ({0}) not in [0, 0.30]")]
    InvalidReserveFraction(f64),
    #[error("invalid: ab_d_multiplier_floor ({0}) not in [0, 1]")]
    InvalidAbDMultiplierFloor(f64),
}

impl ComplianceConfig {
    /// Load from a YAML file (used at service start).
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ComplianceConfigError> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&content)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Parse from a YAML string (used in tests).
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ComplianceConfigError> {
        let cfg: Self = serde_yaml::from_str(yaml)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Validate all invariants per Source §4, §12, §15 and Compliance CI gate.
    pub fn validate(&self) -> Result<(), ComplianceConfigError> {
        // H_c weights sum to 1.0 within 1e-6.
        let sum: f64 = self.h_c_weights.iter().sum();
        if (sum - 1.0).abs() > 1e-6 {
            return Err(ComplianceConfigError::InvalidWeights(sum));
        }
        for (i, w) in self.h_c_weights.iter().enumerate() {
            if !(*w > 0.0 && *w <= 1.0) {
                return Err(ComplianceConfigError::InvalidWeightComponent(i, *w));
            }
        }

        // φ weights sum to 1.0 within 1e-6.
        let phi_sum: f64 = self.phi_weights.iter().sum();
        if (phi_sum - 1.0).abs() > 1e-6 {
            return Err(ComplianceConfigError::InvalidWeights(phi_sum));
        }

        // H_c warmup_threshold < standard_threshold.
        if self.h_c_warmup_threshold >= self.h_c_standard_threshold {
            return Err(ComplianceConfigError::InvalidHCThresholds(
                self.h_c_warmup_threshold,
                self.h_c_standard_threshold,
            ));
        }

        // Reserve fraction in [0, 0.30].
        if !(0.0..=0.30).contains(&self.reserve_fraction) {
            return Err(ComplianceConfigError::InvalidReserveFraction(
                self.reserve_fraction,
            ));
        }

        // ab_d_multiplier_floor in [0, 1].
        if !(0.0..=1.0).contains(&self.ab_d_multiplier_floor) {
            return Err(ComplianceConfigError::InvalidAbDMultiplierFloor(
                self.ab_d_multiplier_floor,
            ));
        }

        // phi_qualification_threshold in (0, 1].
        if !(0.0..=1.0).contains(&self.phi_qualification_threshold) {
            return Err(ComplianceConfigError::InvalidWeights(
                self.phi_qualification_threshold,
            ));
        }

        Ok(())
    }

    /// API_SUPPORTED_SET — parsed Track A action types.
    pub fn api_supported_set(&self) -> Result<Vec<String>, ComplianceConfigError> {
        Ok(serde_json::from_str(&self.api_supported_set_json)?)
    }

    /// Returns the cap_base for the standard account (Scenario B).
    pub fn cap_base_for(&self, action: crate::action::ActionType) -> u32 {
        use crate::action::ActionType::*;
        match action {
            ConnectionRequest => self.caps.connection_request,
            DirectMessage | SequenceStepSend => self.caps.direct_message,
            Comment => self.caps.comment,
            Like => self.caps.like,
            ProfileEditSubmit => self.caps.profile_edit_submission,
            PostPublish => self.caps.post_publish,
            _ => 0,
        }
    }

    /// Returns the warm-up floor cap for new accounts (Scenario C).
    pub fn warm_up_cap_for(&self, action: crate::action::ActionType) -> u32 {
        use crate::action::ActionType::*;
        match action {
            ConnectionRequest => self.warm_up_floor_caps.connection_request,
            DirectMessage | SequenceStepSend => self.warm_up_floor_caps.direct_message,
            Comment => self.warm_up_floor_caps.comment,
            Like => self.warm_up_floor_caps.like,
            ProfileEditSubmit => self.warm_up_floor_caps.profile_edit_submission,
            PostPublish => self.warm_up_floor_caps.post_publish,
            _ => 0,
        }
    }

    /// Returns the minimum spacing for this action type (in seconds).
    pub fn min_spacing_for(&self, action: crate::action::ActionType) -> u64 {
        use crate::action::ActionType::*;
        match action {
            ConnectionRequest => self.min_spacing_seconds.connection_request,
            DirectMessage | SequenceStepSend => self.min_spacing_seconds.direct_message,
            Comment => self.min_spacing_seconds.comment,
            Like => self.min_spacing_seconds.like,
            _ => 0,
        }
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_validates() {
        let cfg = ComplianceConfig::default();
        cfg.validate().expect("default must validate");
    }

    #[test]
    fn weights_must_sum_to_one() {
        let mut cfg = ComplianceConfig::default();
        cfg.h_c_weights = [0.4, 0.4, 0.1, 0.1]; // sums to 1.0 OK
        cfg.validate().unwrap();

        cfg.h_c_weights = [0.5, 0.3, 0.1, 0.1]; // sums to 1.0
        cfg.validate().unwrap();

        cfg.h_c_weights = [0.5, 0.3, 0.1, 0.2]; // sums to 1.1 BAD
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeights(_))
        ));
    }

    #[test]
    fn weight_components_in_unit_interval() {
        let mut cfg = ComplianceConfig::default();

        // A zero weight disables an axis, which the config forbids.
        cfg.h_c_weights = [0.0, 0.5, 0.25, 0.25];
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeightComponent(0, _))
        ));

        // Every vector below sums to 1.0 on purpose: the sum check runs first,
        // so a vector that does not sum to 1.0 would report InvalidWeights and
        // never reach the per-component check being exercised.

        // A weight above 1.0 is rejected.
        cfg.h_c_weights = [0.5, 1.5, -0.5, -0.5];
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeightComponent(1, _))
        ));

        // A negative weight is rejected.
        cfg.h_c_weights = [0.9, 0.9, -0.4, -0.4];
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeightComponent(2, _))
        ));

        // Weights that do not sum to 1.0 are rejected as a sum error.
        cfg.h_c_weights = [0.4, 0.3, 0.2, 0.2];
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeights(_))
        ));

        // When several are wrong, the *first* is reported so the operator is
        // pointed at the leftmost real problem.
        cfg.h_c_weights = [1.5, -0.5, 0.0, 0.0];
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidWeightComponent(0, _))
        ));
    }

    #[test]
    fn warmup_less_than_standard() {
        let mut cfg = ComplianceConfig::default();
        cfg.h_c_warmup_threshold = 0.8;
        cfg.h_c_standard_threshold = 0.7;
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidHCThresholds(_, _))
        ));
    }

    #[test]
    fn reserve_fraction_in_range() {
        let mut cfg = ComplianceConfig::default();
        cfg.reserve_fraction = 0.5;
        assert!(matches!(
            cfg.validate(),
            Err(ComplianceConfigError::InvalidReserveFraction(_))
        ));
    }

    #[test]
    fn cap_lookup_by_action_type() {
        let cfg = ComplianceConfig::default();
        assert_eq!(
            cfg.cap_base_for(crate::action::ActionType::ConnectionRequest),
            18
        );
        assert_eq!(
            cfg.warm_up_cap_for(crate::action::ActionType::DirectMessage),
            6
        );
        assert_eq!(cfg.min_spacing_for(crate::action::ActionType::Comment), 45);
    }

    #[test]
    fn api_supported_set_parses() {
        let cfg = ComplianceConfig::default();
        let set = cfg.api_supported_set().unwrap();
        assert!(set.contains(&"organization_page_post_publish".to_string()));
    }
}
