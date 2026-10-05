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
use crate::{AccountState, CandidateAction};

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
        _account: &AccountState,
        _deps: &GovernorDeps,
    ) -> GuardResult {
        // Tier-1 actions don't consume daily quota.
        if action.action_type.auto_execute_by_default() {
            return GuardResult::pass();
        }

        let cap = ab_d::ab_d(
            action.action_type,
            action_metadata_h_c(action),
            &self.deps.config,
        );
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

/// Read the H_c value from action metadata, falling back to 0 if missing.
fn action_metadata_h_c(action: &CandidateAction) -> f64 {
    action
        .metadata
        .get("h_c")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_extracts_h_c_from_metadata() {
        let mut action = crate::CandidateAction::dummy_for_tests();
        action.metadata.insert("h_c".to_string(), "0.8".to_string());
        assert!((action_metadata_h_c(&action) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn helper_defaults_to_zero() {
        let action = crate::CandidateAction::dummy_for_tests();
        assert_eq!(action_metadata_h_c(&action), 0.0);
    }
}
