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

#[derive(Debug, Clone)]
pub struct RateLimiter {
    limits: HashMap<String, EndpointLimit>,
    /// Per-endpoint counter (UTC date)
    counters: Mutex<HashMap<String, u32>>,
}

impl RateLimiter {
    pub fn new(limits: Vec<EndpointLimit>) -> Self {
        Self {
            limits: limits.into_iter().map(|l| (l.endpoint.clone(), l)).collect(),
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

        let mut counters = self.counters.lock().unwrap();
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
        self.counters.lock().unwrap().clear();
    }

    /// Current usage snapshot for observability.
    pub fn snapshot(&self) -> HashMap<String, u32> {
        self.counters.lock().unwrap().clone()
    }
}

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
        l.check_and_increment("ugc_post").unwrap();
        l.reset();
        assert_eq!(l.snapshot().get("ugc_post").copied().unwrap_or(0), 0);
    }
}
