//! Per-(member, route) rate limit middleware.
//!
//! F-AUDIT-05: this middleware was present but (a) never layered onto the
//! router, and (b) constructed a brand-new `RateLimiter` — and therefore a
//! brand-new, empty counter map — on *every* request. Even had it been
//! routed, no limit could ever be reached. It now keeps a limiter in
//! `AppState` so the window survives across requests.
//!
//! Keying: F-AUDIT-06, the key was previously read from the client-supplied
//! `x-member-id` header, which is trivially spoofable and lets an attacker
//! evade the limit by rotating the header. The key now comes from the
//! `AuthenticatedUser` extension that `require_auth` inserts, falling back to
//! the peer address for unauthenticated traffic.
//!
//! Sizing: Technical Design Spec §17 requires `X-RateLimit-Limit`,
//! `X-RateLimit-Remaining` and `X-RateLimit-Reset` on responses, and
//! `Retry-After` on 429. All four are emitted here.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::sync::Mutex;

use crate::middleware::auth::AuthenticatedUser;
use crate::state::AppState;

/// Sliding-window per-key counter.
#[derive(Default, Clone, Copy)]
struct Counter {
    window_start_ms: i64,
    count: u64,
}

/// Fixed-window counter map.
///
/// Note on scalability: this is per-process and in-memory, so the effective
/// limit is `N × max_requests` across `N` gateway replicas and a restart
/// resets every window. Backend Design Concept §11 (Technical Design Spec)
/// requires the quota path to be durable/Redis-backed; the action-count guard
/// (compliance-governor `daily_cap`) is the authoritative quota and *is*
/// Redis-backed, so this middleware is a coarse abuse guard only, not the
/// compliance quota. Moving it to Redis is tracked as a known limitation.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Counter>>>,
    window_ms: i64,
    max_requests: u64,
}

impl RateLimiter {
    pub fn new(window_ms: i64, max_requests: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            window_ms,
            max_requests,
        }
    }

    /// Returns the decision plus the remaining allowance and reset time.
    pub async fn check(&self, key: &str, now_ms: i64) -> (RateDecision, u64, i64) {
        let mut g = self.inner.lock().await;
        let entry = g.entry(key.to_string()).or_default();
        if now_ms - entry.window_start_ms >= self.window_ms {
            entry.window_start_ms = now_ms;
            entry.count = 0;
        }
        if entry.count >= self.max_requests {
            let reset_ms = entry.window_start_ms + self.window_ms;
            return (RateDecision::Deny, 0, reset_ms);
        }
        entry.count += 1;
        let remaining = self.max_requests.saturating_sub(entry.count);
        let reset_ms = entry.window_start_ms + self.window_ms;
        (RateDecision::Allow, remaining, reset_ms)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RateDecision {
    Allow,
    Deny,
}

pub async fn rate_limit(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Prefer the JWT-derived identity; fall back to peer address. Never trust
    // a client-supplied header for the key.
    let key = match req.extensions().get::<AuthenticatedUser>() {
        Some(user) => format!("member:{}", user.claims.sub),
        None => match connect_info {
            Some(ConnectInfo(addr)) => format!("ip:{}", addr.ip()),
            None => "ip:unknown".to_string(),
        },
    };

    let now_ms = chrono::Utc::now().timestamp_millis();
    let max = state.config().rate_limit_per_minute as u64;
    let (decision, remaining, reset_ms) = state.rate_limiter().check(&key, now_ms).await;

    if decision == RateDecision::Deny {
        let retry_after = ((reset_ms - now_ms).max(0) as f64 / 1000.0).ceil() as u64;
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({
                "error": { "code": "rate_limited", "message": "request quota exceeded" }
            })),
        )
            .into_response();
        set_i64(&mut response, header::RETRY_AFTER.as_str(), retry_after as i64);
        set_i64(&mut response, "x-ratelimit-limit", max as i64);
        set_i64(&mut response, "x-ratelimit-remaining", 0);
        set_i64(&mut response, "x-ratelimit-reset", (reset_ms / 1000).max(0));
        return Ok(response);
    }

    let mut response = next.run(req).await;
    set_i64(&mut response, "x-ratelimit-limit", max as i64);
    set_i64(&mut response, "x-ratelimit-remaining", remaining as i64);
    set_i64(&mut response, "x-ratelimit-reset", (reset_ms / 1000).max(0));
    Ok(response)
}

fn set_i64(response: &mut Response, name: &str, value: i64) {
    if let Ok(n) = axum::http::HeaderName::from_bytes(name.as_bytes()) {
        if let Ok(v) = HeaderValue::from_str(&value.to_string()) {
            response.headers_mut().insert(n, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn limiter_denies_after_max() {
        let rl = RateLimiter::new(60_000, 3);
        for _ in 0..3 {
            let (d, _, _) = rl.check("k", 1_000).await;
            assert_eq!(d, RateDecision::Allow);
        }
        let (d, remaining, _) = rl.check("k", 1_000).await;
        assert_eq!(d, RateDecision::Deny);
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn limiter_resets_after_window() {
        let rl = RateLimiter::new(1_000, 1);
        let (d, _, _) = rl.check("k", 0).await;
        assert_eq!(d, RateDecision::Allow);
        let (d, _, _) = rl.check("k", 500).await;
        assert_eq!(d, RateDecision::Deny);
        // window elapsed
        let (d, remaining, _) = rl.check("k", 1_500).await;
        assert_eq!(d, RateDecision::Allow);
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn limiter_is_per_key() {
        let rl = RateLimiter::new(60_000, 1);
        let (a, _, _) = rl.check("a", 0).await;
        let (b, _, _) = rl.check("b", 0).await;
        assert_eq!(a, RateDecision::Allow);
        assert_eq!(b, RateDecision::Allow);
    }
}
