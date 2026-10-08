//! Guard 3 — Duplicate target (Source §11 guard 3).
//!
//! No active sequence to the same contact; no recently completed (within
//! 30-day cooldown) sequence to the same contact.

use async_trait::async_trait;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct DuplicateTargetGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl DuplicateTargetGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for DuplicateTargetGuard {
    fn name(&self) -> &'static str {
        "duplicate_target"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        _account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult {
        // Only applies to outreach-type actions targeting a contact.
        let contact_id = match action.target_contact_id {
            Some(c) => c,
            None => return GuardResult::pass(),
        };

        let cooldown_days = deps.config.duplicate_target_cooldown_days;

        // Active sequence for the same contact? Unique constraint at DB level,
        // but we double-check.
        let active_query = sqlx::query_scalar::<_, i64>(
            r#"
            -- The table is `lcc.sequences` and the column is `state`
            -- (lcc.sequence_state), not `status`.
            SELECT COUNT(*) FROM lcc.sequences
            WHERE member_id = $1
              AND contact_id = $2
              AND state = 'active'
            "#,
        )
        .bind(action.member_id)
        .bind(contact_id)
        .fetch_one(&deps.db)
        .await;

        let active_count = match active_query {
            Ok(n) => n,
            Err(e) => {
                tracing::error!(error = %e, "duplicate_target: db error");
                return GuardResult::fail("duplicate_target_check_failed");
            }
        };
        if active_count > 0 {
            return GuardResult::fail("duplicate_target: active_sequence_exists_for_contact");
        }

        // Recently completed (within cooldown)?
        let recent_query = sqlx::query_scalar::<_, i64>(
            r#"
            -- lcc.sequence_state has five values
            -- (active|paused|completed|abandoned|replied). The guard's
            -- `completed_no_reply` / `completed_engaged` distinction has no
            -- column to live in, so both collapse to the single `completed`
            -- state; what the reply did not change is `completed_at`, which is
            -- the real "when did this sequence end" column.
            SELECT COUNT(*) FROM lcc.sequences
            WHERE member_id = $1
              AND contact_id = $2
              AND state IN ('completed', 'abandoned', 'replied')
              AND completed_at > now() - ($3 || ' days')::interval
            "#,
        )
        .bind(action.member_id)
        .bind(contact_id)
        .bind(cooldown_days.to_string())
        .fetch_one(&deps.db)
        .await;

        let recent_count = match recent_query {
            Ok(n) => n,
            Err(_) => return GuardResult::fail("duplicate_target_check_failed"),
        };
        if recent_count > 0 {
            return GuardResult::fail(format!(
                "duplicate_target: contact_contacted_within_{cooldown_days}d_cooldown"
            ));
        }

        GuardResult::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Needs a Tokio runtime: the placeholder deps build a sqlx pool, and
    // `connect_lazy` spawns that pool's background task.
    #[tokio::test]
    async fn guard_name() {
        let g = DuplicateTargetGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "duplicate_target");
    }
}
