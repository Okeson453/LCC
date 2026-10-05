//! The 8-guard stack. Each guard is sequential; ALL must pass (axiom 3:
//! partial-pass = deny).
//!
//! Source: Backend Design Concept §27; Technical Design Spec §11.

pub mod account_health;
pub mod approval_state;
pub mod cooldown;
pub mod daily_cap;
pub mod duplicate_target;
pub mod grounding;
pub mod restriction_flag;
pub mod session_pacing;

use async_trait::async_trait;
use lcc_compliance::action::{ActionType, RiskTier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

/// GuardResult — outcome of one guard check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardResult {
    pub passed: bool,
    pub reason: String,
}

impl GuardResult {
    pub fn pass() -> Self {
        Self {
            passed: true,
            reason: "ok".into(),
        }
    }

    pub fn fail(reason: impl Into<String>) -> Self {
        Self {
            passed: false,
            reason: reason.into(),
        }
    }
}

/// Guard trait — implemented by each of the 8 guards.
#[async_trait]
pub trait Guard: Send + Sync {
    fn name(&self) -> &'static str;

    async fn check(
        &self,
        action: &CandidateAction,
        account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult;
}

/// Build the canonical 8-guard stack in spec order:
/// daily_cap → cooldown → duplicate_target → account_health → grounding →
/// restriction_flag → approval_state → session_pacing.
pub fn build_guard_stack(deps: &GovernorDeps) -> Vec<Box<dyn Guard>> {
    vec![
        Box::new(daily_cap::DailyCapGuard::new(deps.clone())),
        Box::new(cooldown::CooldownGuard::new(deps.clone())),
        Box::new(duplicate_target::DuplicateTargetGuard::new(deps.clone())),
        Box::new(account_health::AccountHealthGuard::new(deps.clone())),
        Box::new(grounding::GroundingGuard::new(deps.clone())),
        Box::new(restriction_flag::RestrictionFlagGuard::new(deps.clone())),
        Box::new(approval_state::ApprovalStateGuard::new(deps.clone())),
        Box::new(session_pacing::SessionPacingGuard::new(deps.clone())),
    ]
}

/// Helper: returns the action type's risk tier from the candidate.
pub fn action_risk_tier(action_type: ActionType) -> RiskTier {
    action_type.risk_tier()
}

/// Helper: build a structured reason for a daily-cap denial.
pub fn daily_cap_denial_reason(
    action_type: ActionType,
    used: u32,
    cap: u32,
    next_window_ms: u64,
) -> String {
    format!(
        "daily_cap_exceeded: {action_type:?} used={used} cap={cap} next_window_ms={next_window_ms}"
    )
}

/// Helper: build a reason for grounding failure.
pub fn grounding_denial_reason(kb_ref_count: usize) -> String {
    format!("grounding_missing: kb_refs={kb_ref_count} (axiom 5 requires >=1)")
}

/// Helper: build a reason for restriction flag failure.
pub fn restriction_denial_reason(restricted_reason: &str) -> String {
    format!("account_restricted: {restricted_reason}")
}

/// Helper: build a reason for cooldown failure.
pub fn cooldown_denial_reason(
    action_type: ActionType,
    last_action_ms: i64,
    min_spacing_ms: i64,
) -> String {
    format!(
        "cooldown_violated: {action_type:?} last_action_ms_ago={last_action_ms} min_spacing_ms={min_spacing_ms}"
    )
}

/// Helper: format a UUID for log messages.
pub fn short_id(id: &Uuid) -> String {
    id.to_string()[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_result_helpers() {
        let pass = GuardResult::pass();
        assert!(pass.passed);

        let fail = GuardResult::fail("test reason");
        assert!(!fail.passed);
        assert_eq!(fail.reason, "test reason");
    }

    #[test]
    fn denial_reason_helpers() {
        let s = daily_cap_denial_reason(ActionType::ConnectionRequest, 19, 18, 3_600_000);
        assert!(s.contains("daily_cap_exceeded"));
        assert!(s.contains("19"));
        assert!(s.contains("18"));
    }

    #[test]
    fn grounding_reason_mentions_axiom() {
        let s = grounding_denial_reason(0);
        assert!(s.contains("axiom 5"));
    }

    #[test]
    fn restriction_reason() {
        let s = restriction_denial_reason("captcha");
        assert!(s.contains("captcha"));
    }
}
