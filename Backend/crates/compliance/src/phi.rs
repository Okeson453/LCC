//! Opportunity Fit Score (φ) — Source Technical Design Spec §6.3.
//!
//! ```math
//! φ = 0.4 * S_skill + 0.25 * S_seniority + 0.20 * S_geo + 0.15 * S_comp
//! ```
//!
//! Client-track scoring substitutes `S_trigger_recency` for `S_comp`.
//!
//! The `phi_weights` are loaded from the active `compliance_config_version`
//! (Source axiom 9).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhiInputs {
    pub skill: f64,
    pub seniority: f64,
    pub geo: f64,
    pub comp: f64,            // job-track
    pub trigger_recency: f64, // client-track
    pub goal_mode: GoalMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalMode {
    JobHunting,
    ClientAcquisition,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhiComponents {
    pub skill: f64,
    pub seniority: f64,
    pub geo: f64,
    pub comp: f64,
    pub trigger_recency: f64,
    pub goal_mode: GoalMode,
}

fn clamp01(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}

/// Compute φ for the given inputs and weights.
///
/// Weights are length 4 in [w_skill, w_seniority, w_geo, w_comp]. For
/// client-track, `w_comp` is reassigned to `S_trigger_recency`.
pub fn phi(inputs: &PhiInputs, weights: &[f64; 4]) -> f64 {
    let s_skill = clamp01(inputs.skill);
    let s_seniority = clamp01(inputs.seniority);
    let s_geo = clamp01(inputs.geo);
    let s_comp = clamp01(inputs.comp);
    let s_trigger = clamp01(inputs.trigger_recency);

    let score = match inputs.goal_mode {
        GoalMode::JobHunting | GoalMode::Hybrid => {
            weights[0] * s_skill
                + weights[1] * s_seniority
                + weights[2] * s_geo
                + weights[3] * s_comp
        }
        GoalMode::ClientAcquisition => {
            weights[0] * s_skill
                + weights[1] * s_seniority
                + weights[2] * s_geo
                + weights[3] * s_trigger
        }
    };

    clamp01(score)
}

pub fn phi_with_components(inputs: &PhiInputs, weights: &[f64; 4]) -> (f64, PhiComponents) {
    let score = phi(inputs, weights);
    (
        score,
        PhiComponents {
            skill: clamp01(inputs.skill),
            seniority: clamp01(inputs.seniority),
            geo: clamp01(inputs.geo),
            comp: clamp01(inputs.comp),
            trigger_recency: clamp01(inputs.trigger_recency),
            goal_mode: inputs.goal_mode,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_weights() -> [f64; 4] {
        [0.40, 0.25, 0.20, 0.15]
    }

    #[test]
    fn job_track_perfect_fit() {
        let inputs = PhiInputs {
            skill: 1.0,
            seniority: 1.0,
            geo: 1.0,
            comp: 1.0,
            trigger_recency: 1.0,
            goal_mode: GoalMode::JobHunting,
        };
        let score = phi(&inputs, &default_weights());
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn job_track_zero_fit() {
        let inputs = PhiInputs {
            skill: 0.0,
            seniority: 0.0,
            geo: 0.0,
            comp: 0.0,
            trigger_recency: 0.0,
            goal_mode: GoalMode::JobHunting,
        };
        assert_eq!(phi(&inputs, &default_weights()), 0.0);
    }

    #[test]
    fn client_track_uses_trigger_recency() {
        let inputs = PhiInputs {
            skill: 0.0,
            seniority: 0.0,
            geo: 0.0,
            comp: 0.0,
            trigger_recency: 1.0, // only this is 1.0
            goal_mode: GoalMode::ClientAcquisition,
        };
        // 0 + 0 + 0 + 0.15 * 1.0 = 0.15
        let score = phi(&inputs, &default_weights());
        assert!((score - 0.15).abs() < 1e-9);
    }

    #[test]
    fn hybrid_uses_comp() {
        let inputs = PhiInputs {
            skill: 0.0,
            seniority: 0.0,
            geo: 0.0,
            comp: 1.0,
            trigger_recency: 0.0,
            goal_mode: GoalMode::Hybrid,
        };
        let score = phi(&inputs, &default_weights());
        assert!((score - 0.15).abs() < 1e-9);
    }

    #[test]
    fn components_returned() {
        let inputs = PhiInputs {
            skill: 0.8,
            seniority: 0.6,
            geo: 0.4,
            comp: 0.2,
            trigger_recency: 0.5,
            goal_mode: GoalMode::JobHunting,
        };
        let (score, comps) = phi_with_components(&inputs, &default_weights());
        let expected = 0.40 * 0.8 + 0.25 * 0.6 + 0.20 * 0.4 + 0.15 * 0.2;
        assert!((score - expected).abs() < 1e-9);
        assert_eq!(comps.skill, 0.8);
        assert_eq!(comps.seniority, 0.6);
    }

    #[test]
    fn nan_inputs_clamped() {
        let inputs = PhiInputs {
            skill: f64::NAN,
            seniority: 0.5,
            geo: 0.5,
            comp: 0.5,
            trigger_recency: 0.5,
            goal_mode: GoalMode::JobHunting,
        };
        let score = phi(&inputs, &default_weights());
        // NaN skill → 0.0; rest = 0.5
        // 0 + 0.25*0.5 + 0.20*0.5 + 0.15*0.5 = 0.125 + 0.10 + 0.075 = 0.30
        assert!((score - 0.30).abs() < 1e-9);
    }
}
