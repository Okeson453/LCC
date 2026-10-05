//! Per-endpoint rate-limit awareness for LinkedIn clients.
//!
//! Tracks our local counters and warns when approaching the documented limits.
//! Note: these are application-side observability; the platform's actual
//! 429s come from LinkedIn itself.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointLimit {
    pub endpoint: String,
    pub daily_quota: u32,
    pub burst_per_minute: u32,
}

// Deliberately not `Clone`: the counter map is shared mutable state, and every
// consumer holds a `Arc<RateLimiter>` rather than copying the limiter.
#[derive(Debug)]
pub struct RateLimiter {
    limits: HashMap<String, EndpointLimit>,
    /// Per-endpoint counter (UTC date)
    counters: Mutex<HashMap<String, u32>>,
}

/// Locks the counter map, converting a poisoned mutex into an owned error
/// instead of unwrapping. A panic while the lock was held leaves the map in an
/// unknown state, so the caller must treat this as a limiter failure, not as a
/// free pass.
fn lock_counters(
    counters: &Mutex<HashMap<String, u32>>,
) -> Result<std::sync::MutexGuard<'_, HashMap<String, u32>>, String> {
    counters
        .lock()
        .map_err(|_| "rate limiter counters poisoned".to_string())
}

impl RateLimiter {
    pub fn new(limits: Vec<EndpointLimit>) -> Self {
        Self {
            limits: limits
                .into_iter()
                .map(|l| (l.endpoint.clone(), l))
                .collect(),
            counters: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_default_linkedin_limits() -> Self {
        Self::new(vec![
            EndpointLimit {
                endpoint: "ugc_post".into(),
                daily_quota: 100,
                burst_per_minute: 10,
            },
            EndpointLimit {
                endpoint: "userinfo".into(),
                daily_quota: 1000,
                burst_per_minute: 60,
            },
            EndpointLimit {
                endpoint: "jobs_search".into(),
                daily_quota: 500,
                burst_per_minute: 30,
            },
        ])
    }

    /// Check + increment the counter for an endpoint.
    /// Returns Ok(()) if under quota, Err(quota exceeded) if at the limit.
    pub fn check_and_increment(&self, endpoint: &str) -> Result<(), String> {
        let limit = self
            .limits
            .get(endpoint)
            .ok_or_else(|| format!("unknown endpoint: {endpoint}"))?;

        let mut counters = lock_counters(&self.counters)?;
        let count = counters.entry(endpoint.to_string()).or_insert(0);
        if *count >= limit.daily_quota {
            return Err(format!(
                "{endpoint} daily quota exceeded ({}/{})",
                *count, limit.daily_quota
            ));
        }
        *count += 1;
        Ok(())
    }

    /// Reset counters — call at UTC midnight via cron.
    pub fn reset(&self) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.clear();
        } else {
            tracing::error!("rate limiter counters poisoned during reset");
        }
    }

    /// Current usage snapshot for observability.
    pub fn snapshot(&self) -> HashMap<String, u32> {
        match self.counters.lock() {
            Ok(counters) => counters.clone(),
            Err(_) => {
                tracing::error!("rate limiter counters poisoned during snapshot");
                HashMap::new()
            }
        }
    }
}

// `poisoned_counters_fail_closed` panics on purpose, inside a spawned thread,
// to leave the counter mutex poisoned. The production `panic` deny does not
// apply to a test that is deliberately causing a panic.
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_until_quota() {
        let l = RateLimiter::new(vec![EndpointLimit {
            endpoint: "test".into(),
            daily_quota: 3,
            burst_per_minute: 100,
        }]);
        assert!(l.check_and_increment("test").is_ok());
        assert!(l.check_and_increment("test").is_ok());
        assert!(l.check_and_increment("test").is_ok());
        assert!(l.check_and_increment("test").is_err());
    }

    #[test]
    fn unknown_endpoint_rejected() {
        let l = RateLimiter::new(vec![]);
        assert!(l.check_and_increment("nope").is_err());
    }

    #[test]
    fn reset_clears() {
        let l = RateLimiter::with_default_linkedin_limits();
        assert!(l.check_and_increment("ugc_post").is_ok());
        l.reset();
        assert_eq!(l.snapshot().get("ugc_post").copied().unwrap_or(0), 0);
    }

    #[test]
    fn poisoned_counters_fail_closed() {
        use std::sync::Arc;

        let limiter = Arc::new(RateLimiter::with_default_linkedin_limits());
        let writer = Arc::clone(&limiter);
        // Panic while holding the lock so the mutex is left poisoned.
        let _ = std::thread::spawn(move || {
            if let Ok(mut c) = writer.counters.lock() {
                c.insert("ugc_post".to_string(), 1);
                panic!("poison the counter lock");
            }
        })
        .join();

        // The limiter must fail closed rather than panic or hand out quota.
        assert!(limiter.check_and_increment("ugc_post").is_err());
        assert!(limiter.snapshot().is_empty());
    }
}
