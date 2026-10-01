//! Error type for the Integration Gateway service.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("lcc error: {0}")]
    Lcc(#[from] lcc_error::LccError),
    #[error("permit error: {0}")]
    Permit(#[from] crate::permit::verifier::PermitError),
    #[error("idempotency error: {0}")]
    Idempotency(#[from] crate::idempotency::IdempotencyError),
    #[error("circuit breaker open for {provider}:{endpoint}")]
    CircuitOpen { provider: String, endpoint: String },
    #[error("circuit breaker error: {0}")]
    CircuitBreaker(String),    #[error("vault error: {0}")]
    Vault(String),
    /// F-72: carries the original HTTP status so the caller can distinguish
    /// `429` (rate-limit) from `401/403` (auth revoked) and pull the matching
    /// `RestrictionSignal` without losing info.
    #[error("linkedin api status={status}: {message}")]
    LinkedIn { status: u16, message: String },
    #[error("linkedin api error: {0}")]
    LinkedInLegacy(String),
    #[error("track b ws error: {0}")]
    TrackB(String),
    #[error("config error: {0}")]
    Config(String),
    /// A caller-supplied action payload that does not deserialize into the
    /// target request shape is a client error, not an upstream failure.
    #[error("malformed action payload: {0}")]
    Payload(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    /// Connection-pool failures (deadpool wraps the `redis::RedisError` it is
    /// built on, so a pool checkout error is not convertible via `Redis`).
    #[error("redis pool error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl IntegrationError {
    /// Build a `LinkedIn` variant with explicit status (F-72).
    pub fn linkedin_status(status: u16, msg: impl Into<String>) -> Self {
        Self::LinkedIn {
            status,
            message: msg.into(),
        }
    }

    /// Build a `LinkedIn` variant from any string (no known status).
    pub fn linkedin_unknown(msg: impl Into<String>) -> Self {
        Self::LinkedInLegacy(msg.into())
    }

    /// Extract an HTTP status if the error carries one (0 means unknown).
    pub fn http_status(&self) -> u16 {
        match self {
            IntegrationError::LinkedIn { status, .. } => *status,
            _ => 0,
        }
    }

    /// Extract the error body text (LinkedIn error message).
    pub fn body_text(&self) -> &str {
        match self {
            IntegrationError::LinkedIn { message, .. } => message.as_str(),
            IntegrationError::LinkedInLegacy(s) => s.as_str(),
            _ => "",
        }
    }
}

impl From<lcc_security::vault::VaultError> for IntegrationError {
    fn from(e: lcc_security::vault::VaultError) -> Self {
        IntegrationError::Vault(e.to_string())
    }
}

impl From<lcc_integrations::track_a::TrackAError> for IntegrationError {
    fn from(e: lcc_integrations::track_a::TrackAError) -> Self {
        // TrackAError doesn't carry status; propagate as legacy variant.
        IntegrationError::LinkedInLegacy(e.to_string())
    }
}

impl From<lcc_integrations::track_b::TrackBError> for IntegrationError {
    fn from(e: lcc_integrations::track_b::TrackBError) -> Self {
        IntegrationError::TrackB(e.to_string())
    }
}

impl From<crate::circuit_breaker::CircuitBreakerError> for IntegrationError {
    fn from(e: crate::circuit_breaker::CircuitBreakerError) -> Self {
        // A pool/connection failure surfaces through the circuit breaker, so
        // unwrap the nested redis error where one is present.
        match e {
            crate::circuit_breaker::CircuitBreakerError::Redis(inner) => {
                IntegrationError::Redis(inner)
            }
            other => IntegrationError::CircuitBreaker(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permit_error_displays() {
        let e = IntegrationError::Permit(crate::permit::verifier::PermitError::KeyNotLoaded);
        assert!(e.to_string().contains("permit"));
    }

    #[test]
    fn linkedin_carries_status() {
        let e = IntegrationError::linkedin_status(429, "rate-limited");
        assert_eq!(e.http_status(), 429);
        assert_eq!(e.body_text(), "rate-limited");
    }

    #[test]
    fn legacy_carries_status_zero() {
        let e = IntegrationError::linkedin_unknown("oops");
        assert_eq!(e.http_status(), 0);
        assert_eq!(e.body_text(), "oops");
    }
}
