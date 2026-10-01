//! Circuit breaker — CLOSED → OPEN → HALF_OPEN state machine.
//!
//! State stored in Redis at keys:
//! - `cb:{provider}:{endpoint}:state` — string: "closed" | "open" | "half_open"
//! - `cb:{provider}:{endpoint}:fc` — int counter (failure count, 60s TTL)
//! - `cb:{provider}:{endpoint}:ts` — int (first-failure timestamp)
//!
//! Trigger: N=3 consecutive failures of same call within 60s → OPEN.
//! Cooldown: 5 minutes → HALF_OPEN.
//! Probe: 1 call; success → CLOSED, failure → OPEN.
//!
//! ## F-74 fix
//!
//! The original code wrote an integer (failure counter) and read a string
//! (state) to/from the same key, which crashes under `redis::cmd("GET")` with
//! a `WRONGTYPE` error after the first failure. We now use **separate keys**
//! for the counter (`...:fc`) and the state (`...:state`).

use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("circuit breaker open for {provider}:{endpoint}")]
    Open { provider: String, endpoint: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitState {
    fn as_str(self) -> &'static str {
        match self {
            CircuitState::Closed => "closed",
            CircuitState::Open => "open",
            CircuitState::HalfOpen => "half_open",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "open" => CircuitState::Open,
            "half_open" => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub provider: String,
    pub endpoint: String,
    pub failure_threshold: u32,
    pub cooldown_seconds: i64,
}

impl CircuitBreaker {
    pub fn new(provider: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            endpoint: endpoint.into(),
            failure_threshold: 3,
            cooldown_seconds: 300,
        }
    }

    pub fn state_key(&self) -> String {
        format!("cb:{}:{}:state", self.provider, self.endpoint)
    }
    pub fn failure_counter_key(&self) -> String {
        format!("cb:{}:{}:fc", self.provider, self.endpoint)
    }
    pub fn first_failure_ts_key(&self) -> String {
        format!("cb:{}:{}:ts", self.provider, self.endpoint)
    }

    /// Read the current state (string).
    pub async fn read_state(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<CircuitState, CircuitBreakerError> {
        let s: Option<String> = redis.get(self.state_key()).await?;
        Ok(s.as_deref().map(CircuitState::parse).unwrap_or(CircuitState::Closed))
    }

    /// Returns true if the call should be allowed (closed or half-open probe).
    pub async fn should_allow(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<bool, CircuitBreakerError> {
        let state = self.read_state(redis).await?;
        match state {
            CircuitState::Closed => Ok(true),
            CircuitState::Open => Ok(false),
            CircuitState::HalfOpen => Ok(true), // probe
        }
    }

    /// Record a successful call → reset to CLOSED.
    pub async fn record_success(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<(), CircuitBreakerError> {
        let _: () = redis.del(self.state_key()).await?;
        let _: () = redis.del(self.failure_counter_key()).await?;
        let _: () = redis.del(self.first_failure_ts_key()).await?;
        Ok(())
    }

    /// Record a failure → if N consecutive within 60s, transition to OPEN.
    ///
    /// F-74 fix: failure counter is keyed at `...:fc`, state at `...:state`.
    pub async fn record_failure(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<CircuitState, CircuitBreakerError> {
        let fc_key = self.failure_counter_key();
        let ts_key = self.first_failure_ts_key();
        let state_key = self.state_key();

        let failures: u32 = redis.incr(&fc_key, 1).await?;
        if failures == 1 {
            let _: () = redis
                .set_ex(&ts_key, Utc::now().timestamp(), 60)
                .await?;
            let _: () = redis.expire(&fc_key, 60).await?;
        }

        // Reset counter if first-failure timestamp is older than 60s.
        let ts: Option<i64> = redis.get(&ts_key).await?;
        if let Some(ts_val) = ts {
            let now = Utc::now().timestamp();
            if now - ts_val > 60 {
                // Counter is stale; reset.
                let _: () = redis.del(&fc_key).await?;
                let _: () = redis.del(&ts_key).await?;
                let _: () = redis.incr(&fc_key, 1).await?;
                let _: () = redis.set_ex(&ts_key, now, 60).await?;
                let _: () = redis.expire(&fc_key, 60).await?;
                return Ok(CircuitState::Closed);
            }
        }

        if failures >= self.failure_threshold {
            // Transition to OPEN. Use string state at a *different* key.
            let _: () = redis
                .set_ex(&state_key, CircuitState::Open.as_str(), self.cooldown_seconds as u64)
                .await?;
            Ok(CircuitState::Open)
        } else {
            Ok(CircuitState::Closed)
        }
    }

    /// After the cooldown, transition to HALF_OPEN.
    pub async fn try_half_open(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<bool, CircuitBreakerError> {
        let state_key = self.state_key();
        // Reading the state as a string — fails loudly if WRONGTYPE.
        let s: Option<String> = redis.get(&state_key).await?;
        if s.as_deref() == Some(CircuitState::Open.as_str()) {
            // Cooldown elapsed? Key would have been auto-expired by TTL.
            let _: () = redis
                .set_ex(
                    &state_key,
                    CircuitState::HalfOpen.as_str(),
                    self.cooldown_seconds as u64,
                )
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force-reset (admin only). Used by kill-switch during incident response.
    pub async fn force_close(
        &self,
        redis: &mut deadpool_redis::Connection,
    ) -> Result<(), CircuitBreakerError> {
        self.record_success(redis).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_keys_are_separate() {
        // F-74 sanity: state and counter live at different keys (no WRONGTYPE crash).
        let cb = CircuitBreaker::new("linkedin", "ugc_post");
        assert_eq!(cb.state_key(), "cb:linkedin:ugc_post:state");
        assert_eq!(cb.failure_counter_key(), "cb:linkedin:ugc_post:fc");
        assert_eq!(cb.first_failure_ts_key(), "cb:linkedin:ugc_post:ts");
        assert_ne!(cb.state_key(), cb.failure_counter_key());
    }

    #[test]
    fn default_threshold_3() {
        let cb = CircuitBreaker::new("p", "e");
        assert_eq!(cb.failure_threshold, 3);
        assert_eq!(cb.cooldown_seconds, 300);
    }

    #[test]
    fn state_roundtrip() {
        assert_eq!(CircuitState::parse("open"), CircuitState::Open);
        assert_eq!(CircuitState::parse("half_open"), CircuitState::HalfOpen);
        assert_eq!(CircuitState::parse("closed"), CircuitState::Closed);
        assert_eq!(CircuitState::parse(""), CircuitState::Closed);
        assert_eq!(CircuitState::parse("gibberish"), CircuitState::Closed);
    }
}
