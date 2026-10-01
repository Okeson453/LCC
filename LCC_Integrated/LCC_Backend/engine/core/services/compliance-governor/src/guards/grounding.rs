//! Guard 5 — Grounding (Source §11 guard 5; axiom 5).
//!
//! Every action that produces an external-facing artifact must cite ≥1 KB record.
//! Empty grounding on auto-generated idea stubs is blocked at Content Service
//! BEFORE reaching the Governor, but the Governor re-checks as a wire-format
//! guarantee.

use async_trait::async_trait;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct GroundingGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl GroundingGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for GroundingGuard {
    fn name(&self) -> &'static str {
        "grounding"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        _account: &AccountState,
        _deps: &GovernorDeps,
    ) -> GuardResult {
        if action.action_type.requires_grounding() && action.kb_refs.is_empty() {
            return GuardResult::fail(format!(
                "grounding_missing: action={:?} kb_refs=0 (axiom 5 requires >=1)",
                action.action_type
            ));
        }
        GuardResult::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_name() {
        let g = GroundingGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "grounding");
    }
}
