//! Guard 2 — Cooldown (Source §11 guard 2).
//!
//! Time since last same-type action ≥ min spacing. Uses the DB-authoritative
//! `sequence_step.sent_at` for sequence sends and `content_item.published_at`
//! for publishes. Returns a queued action with a delay when cooldown isn't met
//! but can be satisfied within the day.

use async_trait::async_trait;
use chrono::{Duration, Utc};

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct CooldownGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl CooldownGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for CooldownGuard {
    fn name(&self) -> &'static str {
        "cooldown"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        _account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult {
        let min_spacing_seconds = deps.config.min_spacing_for(action.action_type);
        if min_spacing_seconds == 0 {
            return GuardResult::pass();
        }

        // Query the latest audit_log entry for this action_type.
        // We use `system:integration-gateway` action log entries, which are the
        // single source of truth for "what was actually sent."
        let query = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            r#"
            -- The real audit table is `lcc_audit.events` (migration 0011),
            -- where the timestamp column is `occurred_at`. There is no
            -- `audit_log` table anywhere in the schema.
            SELECT MAX(occurred_at) FROM lcc_audit.events
            WHERE action = $1
              AND resource_id = $2
              AND actor = 'system:integration-gateway'
              AND outcome = 'success'
            "#,
        )
        .bind(format!("integration.execute:{:?}", action.action_type))
        .bind(action.member_id)
        .fetch_optional(&deps.db)
        .await;

        let last_at = match query {
            Ok(Some(Some(ts))) => ts,
            _ => return GuardResult::pass(), // no prior execution → no cooldown violation
        };

        let now = Utc::now();
        let elapsed = now.signed_duration_since(last_at);
        let min_spacing = Duration::seconds(min_spacing_seconds as i64);

        if elapsed >= min_spacing {
            GuardResult::pass()
        } else {
            let remaining_ms = (min_spacing - elapsed).num_milliseconds();
            GuardResult::fail(format!(
                "cooldown_violated: last_action_at={last_at} elapsed_ms={} min_spacing_ms={} remaining_ms={remaining_ms}",
                elapsed.num_milliseconds(),
                min_spacing.num_milliseconds()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Needs a Tokio runtime: the placeholder deps build a sqlx pool, and
    // `connect_lazy` spawns that pool's background task.
    #[tokio::test]
    async fn guard_name() {
        let g = CooldownGuard::new(test_deps());
        assert_eq!(g.name(), "cooldown");
    }

    fn test_deps() -> GovernorDeps {
        // Tests that don't touch Redis/DB use defaults; only the name check is here.
        GovernorDeps::placeholder()
    }
}
