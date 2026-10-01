//! Guard 7 — Approval state (Source §11 guard 7).
//!
//! For `requires_approval=true` actions, an Approval row must exist with
//! status = 'approved'. Approval missing entirely → fail-closed (not
//! fail-open). For Tier 1 actions with `auto_execute=true`, no approval is
//! required.

use async_trait::async_trait;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct ApprovalStateGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl ApprovalStateGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for ApprovalStateGuard {
    fn name(&self) -> &'static str {
        "approval_state"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        _account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult {
        // Tier 1 + auto_execute → no approval needed.
        if action.auto_execute && action.action_type.auto_execute_by_default() {
            return GuardResult::pass();
        }

        // If action doesn't require approval, pass.
        if !action.requires_approval {
            return GuardResult::pass();
        }

        // An approval row must exist, be approved, and not be expired.
        //
        // F-AUDIT-20: this previously ran
        //     SELECT status FROM approval
        //     WHERE member_id = $1
        //       AND idempotency_key_match($2) = true
        //       AND status = 'approved'
        // Both the table and the function were wrong:
        //   * `approval` does not exist — the table is `lcc.approvals`
        //     (schemas/migrations/0011_approval_audit.sql);
        //   * `status` is not a column — the enum is `decision`
        //     `lcc.approval_decision`;
        //   * `idempotency_key_match()` is not a function anywhere in the
        //     repo. The source even flagged it as "hypothetical".
        // The query therefore always errored, and the error arm is
        // fail-closed, so **every approval-gated action was denied**. Combined
        // with the same defect in guard 6, the governor could not return
        // Permit for any action at all.
        //
        // The approval record is correlated to the candidate action by
        // `resource_id` — `lcc.approvals.resource_id` is the UUID of the thing
        // being approved (content item, outreach draft, sequence step, ...),
        // which is exactly what `CandidateAction` carries in its typed id
        // fields. `requested_action->>'idempotency_key'` is used as a secondary
        // match so an approval minted for a specific replay key still resolves.
        let row: Result<Option<(String, bool)>, sqlx::Error> = sqlx::query_as(
            r#"
            SELECT decision::TEXT,
                   (expires_at > NOW()) AS unexpired
            FROM lcc.approvals
            WHERE member_id = $1
              AND decision = 'approved'
              AND (
                    resource_id = $2
                 OR requested_action->>'idempotency_key' = $3
                  )
            ORDER BY decided_at DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(action.member_id)
        .bind(action.id)
        .bind(action.idempotency_key.clone())
        .fetch_optional(&deps.db)
        .await;

        match row {
            // Approved but the approval window has closed: fail closed.
            Ok(Some((_, false))) => GuardResult::fail("approval_state: approval_expired"),
            Ok(Some(_)) => GuardResult::pass(),
            // A pending/rejected/withdrawn approval, or none at all. The spec
            // requires fail-closed: an approval record that is *missing
            // entirely* must not be treated as permission.
            Ok(None) => GuardResult::fail("approval_state: no_approved_approval_found"),
            Err(e) => {
                tracing::error!(error = %e, "approval_state: db error");
                GuardResult::fail("approval_state_check_failed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_name() {
        let g = ApprovalStateGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "approval_state");
    }
}
