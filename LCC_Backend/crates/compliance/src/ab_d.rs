//! Action Budget (AB_d) — Source Technical Design Spec §4.
//!
//! AB_d is a *continuous* function of H_c, not a tier ladder. Source §15
//! resolves Scenarios A/B/C: B is the production default, C is auto-applied
//! to new/low-H_c accounts via the H_c interpolation in `ab_d()`. Scenario A
//! is documented only as "what we explicitly rejected."
//!
//! ```math
//! AB_d(action) = floor(CAP_base(action) * min(1, ab_d_multiplier_floor + (1 - ab_d_multiplier_floor) * H_c))
//! ```

use crate::action::ActionType;
use crate::config::ComplianceConfig;

/// Compute AB_d for a given action and account health.
///
/// `h_c` is clamped to [0,1]. The multiplier is the linear interpolation
/// between `ab_d_multiplier_floor` (H_c=0) and 1.0 (H_c=1) — never zero so a
/// brand-new account still gets a minimal, safe action allowance.
pub fn ab_d(action: ActionType, h_c: f64, config: &ComplianceConfig) -> u32 {
    let h_c_clamped = if h_c.is_nan() { 0.0 } else { h_c.clamp(0.0, 1.0) };

    // cap_base for this action at standard (H_c ≥ standard_threshold) is the
    // value in `config.caps`. For warm-up (H_c < warmup_threshold) we use the
    // warm_up_floor_caps. Between them, interpolate linearly.
    let cap_standard = config.cap_base_for(action);
    let cap_warmup = config.warm_up_cap_for(action);
    let (cap_at_hc, _) = interpolate_caps(h_c_clamped, cap_warmup, cap_standard, config);

    let multiplier = config.ab_d_multiplier_floor + (1.0 - config.ab_d_multiplier_floor) * h_c_clamped;
    let multiplier_clamped = multiplier.clamp(config.ab_d_multiplier_floor, 1.0);

    (cap_at_hc as f64 * multiplier_clamped).floor() as u32
}

/// Linearly interpolate between warm-up floor and standard cap based on H_c
/// thresholds (`h_c_warmup_threshold` and `h_c_standard_threshold`).
///
/// Returns the cap and the cap-equivalent daily budget after applying AB_d.
fn interpolate_caps(h_c: f64, cap_warmup: u32, cap_standard: u32, config: &ComplianceConfig) -> (u32, f64) {
    if cap_standard == 0 {
        return (0, 0.0);
    }

    let warmup_t = config.h_c_warmup_threshold;
    let standard_t = config.h_c_standard_threshold;

    let cap = if h_c <= warmup_t {
        cap_warmup
    } else if h_c >= standard_t {
        cap_standard
    } else {
        // Linear interpolation between warmup and standard.
        let t = (h_c - warmup_t) / (standard_t - warmup_t);
        let cap_w = cap_warmup as f64;
        let cap_s = cap_standard as f64;
        (cap_w + (cap_s - cap_w) * t).floor() as u32
    };

    (cap, h_c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ComplianceConfig;

    fn cfg() -> ComplianceConfig {
        ComplianceConfig::default()
    }

    #[test]
    fn zero_hc_returns_warmup_floor_multiplied() {
        let config = cfg();
        let result = ab_d(ActionType::ConnectionRequest, 0.0, &config);
        // warmup floor = 5; multiplier floor = 0.3
        // result = floor(5 * 0.3) = 1
        assert_eq!(result, 1);
    }

    #[test]
    fn full_hc_returns_full_cap() {
        let config = cfg();
        let result = ab_d(ActionType::ConnectionRequest, 1.0, &config);
        // standard cap = 18; multiplier = 1.0
        // result = floor(18 * 1.0) = 18
        assert_eq!(result, 18);
    }

    #[test]
    fn mid_hc_interpolates() {
        let config = cfg();
        // h_c=0.5 → standard_t=0.7, warmup_t=0.4
        // t = (0.5 - 0.4) / (0.7 - 0.4) = 0.333
        // cap = floor(5 + (18 - 5) * 0.333) = floor(9.333) = 9
        // multiplier = 0.3 + 0.7 * 0.5 = 0.65
        // result = floor(9 * 0.65) = 5
        let result = ab_d(ActionType::ConnectionRequest, 0.5, &config);
        assert_eq!(result, 5);
    }

    #[test]
    fn tier1_actions_return_zero() {
        let config = cfg();
        assert_eq!(ab_d(ActionType::ProfileEditDraft, 0.5, &config), 0);
        assert_eq!(ab_d(ActionType::ContentDraft, 0.0, &config), 0);
        assert_eq!(ab_d(ActionType::OpportunityDiscover, 1.0, &config), 0);
    }

    #[test]
    fn nan_hc_uses_zero() {
        let config = cfg();
        let result = ab_d(ActionType::Like, f64::NAN, &config);
        // Behaves like H_c=0: warmup floor = 10, multiplier = 0.3 → 3
        assert_eq!(result, 3);
    }

    #[test]
    fn all_action_types_have_deterministic_budget() {
        let config = cfg();
        for &action in ActionType::ALL {
            let result = ab_d(action, 0.7, &config);
            // Just ensure no panics and result is bounded.
            assert!(result <= 100, "{action:?} budget={result} too high");
        }
    }
}
