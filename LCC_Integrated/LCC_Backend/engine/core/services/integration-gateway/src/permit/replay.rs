//! Permit-token replay protection.
//!
//! ### F-AUDIT-22 — a valid permit was replayable for its entire TTL
//!
//! Every permit carries a `jti` (unique token id) generated at issuance
//! (`crates/compliance/src/permit_token.rs`), but **no verifier anywhere in
//! the repository ever read it**. The declared error variant
//! `PermitError::Replay` was never constructed, and the module doc at
//! `verifier.rs:10` claimed replay was "checked at the dispatcher via Redis"
//! while the dispatcher did not check it.
//!
//! The consequence: an attacker who observes one permit — or any component
//! that retries a request — can resubmit the exact same token for the same
//! action and member and have it accepted, because every other check
//! (signature, `iss`, `aud`, `sub`, `act`, `exp`, `iat`) is deterministic and
//! therefore still passes on the second presentation. Combined with axiom 1's
//! whole purpose — one permit, one execution — this meant a single governed
//! action could be executed many times without re-entering the guard stack.
//!
//! ### Fix
//!
//! `ReplayGuard` records each `jti` in Redis with `SET key 1 NX EX <ttl>`, the
//! atomically-correct primitive for this: only the first caller wins. A second
//! presentation of the same permit loses the race and is rejected. The key TTL
//! matches the permit's own remaining lifetime, so the dedupe entry cannot
//! outlive the token it protects and cannot be evicted early to reopen the
//! window.

use std::time::Duration;

use redis::AsyncCommands;
use thiserror::Error;

/// Redis key namespace for seen permits.
const SEEN_PREFIX: &str = "permit:seen:";

#[derive(Debug, Error)]
pub enum ReplayGuardError {
    #[error("permit_token replay detected (jti={0})")]
    Replay(String),
    #[error("replay store unavailable: {0}")]
    StoreUnavailable(String),
}

/// Single-use enforcement for permit tokens.
#[derive(Clone)]
pub struct ReplayGuard;

impl ReplayGuard {
    /// Claim a permit for one-time use.
    ///
    /// Returns `Ok(())` for the first presentation and
    /// `Err(ReplayGuardError::Replay(_))` for every subsequent one.
    ///
    /// Fails **closed** on a store error: if Redis is unreachable we cannot
    /// prove the permit is unused, so the action is not executed. This
    /// matches the governor's own posture (axiom 1: "any guard fails -> deny,
    /// never partial-permit").
    pub async fn claim(
        redis: &mut redis::aio::ConnectionManager,
        jti: &str,
        ttl: Duration,
    ) -> Result<(), ReplayGuardError> {
        let key = format!("{SEEN_PREFIX}{jti}");
        // Round up so the dedupe entry never expires before the token does.
        let ttl_secs = ttl.as_secs().max(1) + 1;

        let claimed: Option<String> = redis
            .set(&key, "1", redis::SetOptions::default().with_nx().with_ex(ttl_secs))
            .await
            .map_err(|e| ReplayGuardError::StoreUnavailable(e.to_string()))?;

        match claimed {
            // NX succeeded: nobody had seen this jti before.
            Some(_) => Ok(()),
            // NX failed: the key already exists, so this permit is a replay.
            None => Err(ReplayGuardError::Replay(jti.to_string())),
        }
    }

    /// Seconds of validity remaining on a permit, floored at 1 so the dedupe
    /// entry always outlives the token.
    pub fn dedupe_ttl(claims_exp: i64) -> Duration {
        let now = chrono::Utc::now().timestamp();
        Duration::from_secs((claims_exp - now).max(1) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_ttl_never_zero() {
        // An already-expired permit still needs a non-zero TTL so the
        // SET NX cannot become a no-op that silently permits everything.
        let past = chrono::Utc::now().timestamp() - 10;
        assert!(ReplayGuard::dedupe_ttl(past).as_secs() >= 1);
    }

    #[test]
    fn dedupe_ttl_tracks_remaining_lifetime() {
        let exp = chrono::Utc::now().timestamp() + 45;
        let ttl = ReplayGuard::dedupe_ttl(exp);
        assert!(ttl.as_secs() >= 44 && ttl.as_secs() <= 46, "got {}", ttl.as_secs());
    }
}
