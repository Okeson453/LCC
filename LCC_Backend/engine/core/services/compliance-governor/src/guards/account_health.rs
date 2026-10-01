//! Guard 4 — Account health (Source §11 guard 4).
//!
//! H_c ≥ tier minimum. If H_c is stale (>6h), the Governor fetches a fresh
//! value via the scoring-intel gRPC client.

use async_trait::async_trait;
use chrono::{Duration, Utc};

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

pub struct AccountHealthGuard {
    deps: GovernorDeps,
}

impl AccountHealthGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }
}

#[async_trait]
impl Guard for AccountHealthGuard {
    fn name(&self) -> &'static str {
        "account_health"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult {
        // Get the minimum H_c for this action's tier.
        let min_h_c = action.risk_tier.minimum_h_c();

        // NaN / out-of-range → treat as H_c = 0 (Source §32).
        if account.h_c.is_nan() {
            tracing::warn!("h_c is NaN — treating as 0 (fail-safe)");
            return GuardResult::fail("h_c_undefined: NaN treated as 0");
        }

        // If H_c is stale, refresh via scoring-intel.
        let effective_h_c = {
            let stale_threshold = Duration::hours(6);
            let now = Utc::now();
            let stale = now.signed_duration_since(account.h_c_computed_at) > stale_threshold;
            if stale {
                match self.deps.scoring_client.compute_h_c(account, deps).await {
                    Ok(fresh) => fresh,
                    Err(e) => {
                        tracing::warn!(error = %e, "scoring-intel unreachable; using stale H_c");
                        account.h_c
                    }
                }
            } else {
                account.h_c
            }
        };

        if effective_h_c < min_h_c {
            return GuardResult::fail(format!(
                "account_health: h_c={effective_h_c:.3} < required={min_h_c:.3} (tier={:?})",
                action.risk_tier
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
        let g = AccountHealthGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "account_health");
    }
}
