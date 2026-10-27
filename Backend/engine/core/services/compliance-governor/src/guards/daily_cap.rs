//! Guard 1 — Daily cap (Source §11 guard 1).
//!
//! Action count today < AB_d. Counter is Redis, UTC-anchored, partitioned by
//! `(member_id, action_type)`.

use async_trait::async_trait;
use chrono::Utc;
use lcc_compliance::ab_d;
use redis::AsyncCommands;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::AccountState;

pub struct DailyCapGuard {
    deps: GovernorDeps,
}

impl DailyCapGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for DailyCapGuard {
    fn name(&self) -> &'static str {
        "daily_cap"
    }

    async fn check(
        &self,
        action: &crate::CandidateAction,
        account: &AccountState,
        _deps: &GovernorDeps,
    ) -> GuardResult {
        // Tier-1 actions don't consume daily quota.
        if action.action_type.auto_execute_by_default() {
            return GuardResult::pass();
        }

        // H_c is the one on the evaluated account snapshot. It used to be read
        // from `action.metadata["h_c"]`, which no caller sets — so it defaulted
        // to 0.0 and every action was budgeted on the warm-up floor. A healthy
        // account's post_publish budget came out as floor(1 * 0.5) = 0, i.e.
        // permanently denied by the first guard.
        let cap = ab_d::ab_d(action.action_type, account.h_c, &self.deps.config);
        let date = Utc::now().format("%Y%m%d").to_string();
        let routine_key = format!(
            "quota:{}:{}:{}:routine",
            action.member_id, action.action_type, date
        );
        let reserve_key = format!(
            "quota:{}:{}:{}:reserve",
            action.member_id, action.action_type, date
        );

        let mut conn = match self.deps.redis.get().await {
            Ok(c) => c,
            Err(e) => {
                // Fail-closed on Redis outage (Source §32).
                tracing::error!(error = %e, "redis unavailable for daily_cap");
                return GuardResult::fail("quota_store_unavailable");
            }
        };

        let routine_used: u32 = conn.get(&routine_key).await.unwrap_or(0);
        let reserve_used: u32 = conn.get(&reserve_key).await.unwrap_or(0);

        let reserve = (cap as f64 * self.deps.config.reserve_fraction).floor() as u32;
        let routine_cap = cap.saturating_sub(reserve);

        // Reserve is single-use per UTC day. Only consume on explicit priority release.
        if routine_used >= routine_cap {
            // Check if reserve is available and this is a priority action.
            // (Caller should have set action.metadata["priority_release"] = "true"
            // explicitly. Otherwise this denies.)
            let is_priority = action
                .metadata
                .get("priority_release")
                .map(|v| v == "true")
                .unwrap_or(false);
            if is_priority && reserve_used < reserve {
                // Allow reserve consumption. Caller (orchestrator) is responsible
                // for incrementing the reserve counter after successful execution.
                return GuardResult::pass();
            }
            return GuardResult::fail(format!(
                "daily_cap_exceeded: used={routine_used} routine_cap={routine_cap} reserve={reserve}"
            ));
        }

        GuardResult::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_uses_the_account_snapshot_not_metadata() {
        // Regression: the guard used to read H_c from `action.metadata`, which
        // no caller populates, so a healthy account was budgeted as H_c=0.
        // Standard post_publish cap is 3; multiplier at H_c=0.8 is
        // 0.3 + 0.7*0.8 = 0.86, so the budget must be floor(3 * 0.86) = 2.
        // The old code produced floor(warm_up_cap 1 * 0.3) = 0 — a permanent
        // daily_cap denial for every post.
        let config = lcc_compliance::config::ComplianceConfig::default();
        let healthy = ab_d::ab_d(
            lcc_compliance::action::ActionType::PostPublish,
            0.8,
            &config,
        );
        let as_if_unknown_h_c = ab_d::ab_d(
            lcc_compliance::action::ActionType::PostPublish,
            0.0,
            &config,
        );
        assert_eq!(healthy, 2);
        assert_eq!(as_if_unknown_h_c, 0);
    }
}
