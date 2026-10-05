//! Helpers for Compliance Governor guard-stack tests.
//!
//! Each helper constructs a CandidateAction + AccountState for a given scenario.

use lcc_compliance::config::ComplianceConfig;
use lcc_compliance::h_c::{h_c, HCInputs};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCandidateAction {
    pub id: Uuid,
    pub member_id: Uuid,
    pub action_type: String,
    pub risk_tier: u32,
    pub kb_refs: Vec<Uuid>,
    pub idempotency_key: String,
    pub requires_approval: bool,
    pub auto_execute: bool,
    pub target_contact_id: Option<Uuid>,
}

pub struct TestScenario {
    pub action: TestCandidateAction,
    pub h_c_inputs: HCInputs,
    pub weights: [f64; 4],
}

impl TestScenario {
    /// Construct a "happy path" scenario: warm but healthy account, fully
    /// grounded candidate, no approval required.
    pub fn happy_path(member_id: Uuid) -> Self {
        let action = TestCandidateAction {
            id: Uuid::now_v7(),
            member_id,
            action_type: "like".into(),
            risk_tier: 2,
            kb_refs: vec![Uuid::now_v7()],
            idempotency_key: Uuid::now_v7().to_string(),
            requires_approval: false,
            auto_execute: true,
            target_contact_id: None,
        };
        let h_c_inputs = HCInputs {
            acceptance_rate: 0.6,
            reply_rate: 0.4,
            quota_utilization: 0.3,
            tenure_factor: 0.8,
        };
        Self {
            action,
            h_c_inputs,
            weights: ComplianceConfig::default().h_c_weights,
        }
    }

    /// Construct a "low_grounding" scenario: no KB refs, should fail guard 5.
    pub fn low_grounding(member_id: Uuid) -> Self {
        let mut s = Self::happy_path(member_id);
        s.action.kb_refs.clear();
        s.action.action_type = "post_publish".into();
        s.action.risk_tier = 2;
        s
    }

    /// Construct a "low_h_c" scenario: new account, should fail guard 4 for
    /// tier 3+ actions.
    pub fn low_h_c(member_id: Uuid) -> Self {
        let mut s = Self::happy_path(member_id);
        s.h_c_inputs = HCInputs {
            acceptance_rate: 0.05,
            reply_rate: 0.05,
            quota_utilization: 0.05,
            tenure_factor: 0.1,
        };
        s.action.action_type = "connection_request".into();
        s.action.risk_tier = 3;
        s
    }

    /// Construct a "restricted" scenario: account is restricted, should fail
    /// guard 6.
    pub fn restricted(member_id: Uuid) -> Self {
        let mut s = Self::happy_path(member_id);
        s.action.action_type = "direct_message".into();
        s.action.risk_tier = 4;
        s
    }

    pub fn h_c(&self) -> f64 {
        h_c(&self.h_c_inputs, &self.weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_h_c_is_positive() {
        let s = TestScenario::happy_path(Uuid::now_v7());
        assert!(s.h_c() > 0.5);
    }

    #[test]
    fn low_grounding_scenario_has_no_kb_refs() {
        let s = TestScenario::low_grounding(Uuid::now_v7());
        assert!(s.action.kb_refs.is_empty());
    }

    #[test]
    fn low_h_c_scenario_scores_low() {
        let s = TestScenario::low_h_c(Uuid::now_v7());
        assert!(s.h_c() < 0.3);
    }
}
