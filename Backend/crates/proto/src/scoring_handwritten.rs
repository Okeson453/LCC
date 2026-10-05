//! Hand-written tonic message types for the `lcc.v1.intelligence.scoring`
//! package.
//!
//! ### Why hand-written
//! The canonical generation path is `buf generate proto` (or
//! `protoc --rust_out=src/gen/`), which produces real protobuf-encoded
//! binary types via `prost`. In environments where `protoc` is not
//! available (CI build agents that have not yet installed the toolchain,
//! local dev containers, sandboxes), the build falls back to these
//! hand-written message types.
//!
//! The hand-written types are wire-compatible at the **JSON-over-tonic**
//! layer (which is what `tonic::transport::Server` actually serializes
//! by default — see `tonic::codec::JsonCodec`). They are NOT
//! wire-compatible at the raw-protobuf layer; if you need proto-binary
//! transport, run `buf generate proto` and commit the result to
//! `src/gen/`. The CI script `infra/ci/scripts/proto-codegen-check.sh`
//! refuses to merge a PR that doesn't regenerate the proto files.
//!
//! ### Wire layout (matches scoring.proto)
//! ```text
//! service ScoringIntel {
//!   rpc ComputeH_c(ComputeH_cRequest) returns (ComputeH_cResponse);
//!   rpc ComputePhi(ComputePhiRequest) returns (ComputePhiResponse);
//!   rpc PredictReplyProbability(...) returns (...);
//!   rpc CalibrationStatus(...) returns (...);
//! }
//! ```

// The message names below mirror `proto/lcc/v1/intelligence/scoring.proto`
// verbatim. prost derives Rust type names from the proto message names, so
// `buf generate` would produce `ComputeH_cRequest`/`ComputeH_cResponse` with
// exactly these spellings. Renaming the hand-written copies to satisfy
// `non_camel_case_types` would leave the two definitions disagreeing the next
// time someone regenerates, so the lint is suppressed here instead.
#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeH_cRequest {
    pub member_id: String,
    pub compliance_config_version: String,
    pub force_recompute: bool,
    pub trace: Option<TraceContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HCComponents {
    pub acceptance_rate: f64,
    pub reply_rate: f64,
    pub quota_utilization: f64,
    pub tenure_factor: f64,
    pub weights: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct H_cResult {
    pub h_c: f64,
    pub components: Option<HCComponents>,
    pub computed_at_unix_ms: i64,
    pub h_c_undefined: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeH_cResponse {
    pub result: Option<H_cResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FitScoreComponents {
    pub skill: f64,
    pub seniority: f64,
    pub geo: f64,
    pub comp: f64,
    pub trigger_recency: f64,
    pub goal_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputePhiRequest {
    pub member_id: String,
    pub opportunity_id: String,
    pub goal_mode: String,
    pub compliance_config_version: String,
    pub trace: Option<TraceContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputePhiResponse {
    pub opportunity_id: String,
    pub phi: f64,
    pub components: Option<FitScoreComponents>,
    pub eligible: bool,
    pub rule_based_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplyProbabilityFeatures {
    pub member_id: String,
    pub contact_id: String,
    pub mutual_count: u32,
    pub personalization_score: f32,
    pub prior_interaction_flag: bool,
    pub contact_tier: String,
    pub last_interaction_age_days: String,
    pub tag_match_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PredictReplyProbabilityRequest {
    pub features: Option<ReplyProbabilityFeatures>,
    pub trace: Option<TraceContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PredictReplyProbabilityResponse {
    pub rho: f64,
    pub rule_based_fallback: bool,
    pub labeled_send_count: u32,
    pub model_version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationStatusRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationStatusResponse {
    pub h_c_labeled_account_days: u32,
    pub rho_labeled_sends: u32,
    pub h_c_calibrated: bool,
    pub rho_calibrated: bool,
    pub h_c_active_weights_version: String,
    pub rho_active_model_version: String,
    pub last_h_c_fit_at_unix_ms: i64,
    pub last_rho_fit_at_unix_ms: i64,
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h_c_result_serializes() {
        let r = H_cResult {
            h_c: 0.65,
            components: Some(HCComponents {
                acceptance_rate: 0.7,
                reply_rate: 0.5,
                quota_utilization: 0.4,
                tenure_factor: 0.8,
                weights: vec![0.25, 0.25, 0.25, 0.25],
            }),
            computed_at_unix_ms: 1234,
            h_c_undefined: false,
            reason: String::new(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: H_cResult = serde_json::from_str(&json).unwrap();
        assert!((back.h_c - 0.65).abs() < 1e-9);
        assert_eq!(back.components.unwrap().weights.len(), 4);
    }

    #[test]
    fn compute_h_c_request_round_trips() {
        let req = ComputeH_cRequest {
            member_id: "abc".into(),
            compliance_config_version: "v1".into(),
            force_recompute: true,
            trace: Some(TraceContext {
                trace_id: "t-1".into(),
                span_id: Some("s-1".into()),
                parent_span_id: None,
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ComputeH_cRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.member_id, "abc");
        assert!(back.force_recompute);
    }
}
