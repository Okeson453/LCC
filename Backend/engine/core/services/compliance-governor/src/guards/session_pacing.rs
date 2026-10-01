//! Guard 8 — Session pacing (Source §11 guard 8).
//!
//! Randomized jitter since last action in this session ≥ J_min (default 8s).
//! Held in service memory (intra-tick); resets on service restart (safe default).
//!
//! F-69 fix: store is `parking_lot::Mutex` and reads `.get()` directly without
//! `.unwrap()`. The guard stores the latest issued timestamp only when the
//! candidate action passes the rest of the chain — call sites that need to
//! evict/reset use `reset_for_tests` (also panic-free).

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

use super::{Guard, GuardResult};
use crate::state::GovernorDeps;
use crate::{AccountState, CandidateAction};

#[derive(Default)]
struct SessionState {
    by_member_action: HashMap<(uuid::Uuid, String), chrono::DateTime<Utc>>,
}

/// In-process last-action timestamp per (member_id, action_type).
///
/// Wrapped in `parking_lot::Mutex` so deadlocks panic cleanly instead of via
/// `unwrap()` on `StdMutex` (workspace lint: no `unwrap`).
static LAST_SESSION_ACTION: std::sync::LazyLock<parking_lot::Mutex<SessionState>> =
    std::sync::LazyLock::new(|| {
        parking_lot::Mutex::new(SessionState {
            by_member_action: HashMap::new(),
        })
    });

pub struct SessionPacingGuard {
    #[allow(dead_code)]
    deps: GovernorDeps,
}

impl SessionPacingGuard {
    pub fn new(deps: GovernorDeps) -> Self {
        Self { deps }
    }

    /// Reset all session-pacing state — used by integration tests.
    pub fn reset_for_tests() {
        LAST_SESSION_ACTION.lock().by_member_action.clear();
    }
}

#[async_trait]
impl Guard for SessionPacingGuard {
    fn name(&self) -> &'static str {
        "session_pacing"
    }

    async fn check(
        &self,
        action: &CandidateAction,
        _account: &AccountState,
        deps: &GovernorDeps,
    ) -> GuardResult {
        let j_min_seconds = deps.config.session_jitter_min_seconds;
        if j_min_seconds == 0 {
            return GuardResult::pass();
        }

        let key = (action.member_id, action.action_type.to_string());

        // Tier-1 actions are exempt from session pacing (they don't touch the platform).
        if action.action_type.auto_execute_by_default() {
            return GuardResult::pass();
        }

        let mut state = LAST_SESSION_ACTION.lock();
        let now = Utc::now();

        if let Some(ts) = state.by_member_action.get(&key).copied() {
            let elapsed = now.signed_duration_since(ts);
            let min_spacing = chrono::Duration::seconds(j_min_seconds as i64);
            if elapsed < min_spacing {
                // Inject ±20% jitter to avoid thundering-herd patterns.
                let jitter_ms = (elapsed.num_milliseconds() as f64
                    * (rand::random::<f64>() * 0.4 + 0.8))
                    as u64;
                return GuardResult::fail(format!(
                    "session_pacing: jitter_required; last_action={}ms_ago, min={}ms, jitter_inject={}ms",
                    elapsed.num_milliseconds(),
                    j_min_seconds * 1000,
                    jitter_ms
                ));
            }
        }

        state.by_member_action.insert(key, now);
        GuardResult::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_name() {
        let g = SessionPacingGuard::new(GovernorDeps::placeholder());
        assert_eq!(g.name(), "session_pacing");
    }

    #[test]
    fn reset_for_tests_is_no_panic() {
        // This must not panic even if called concurrently or twice in a row.
        SessionPacingGuard::reset_for_tests();
        SessionPacingGuard::reset_for_tests();
    }
}
