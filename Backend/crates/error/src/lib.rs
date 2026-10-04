//! `lcc-error` — Common error type with From-impls for sqlx, tonic, axum, redis.
//!
//! All services in the Rust Core Engine use this error type. It maps cleanly to:
//! - HTTP responses (axum `IntoResponse`)
//! - gRPC responses (`tonic::Status`)
//! - JSON log entries (structured)
//!
//! No `unwrap` / `panic` are exposed publicly — services must handle each variant.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Common error type used across every Rust Core Engine service.
#[derive(Debug, Error)]
pub enum LccError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("not found: {resource_type} {resource_id}")]
    NotFound { resource_type: String, resource_id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("rate limit exceeded")]
    RateLimited { retry_after_ms: u64 },

    #[error("permission token invalid: {0}")]
    PermitTokenInvalid(String),

    #[error("compliance denied: {guard} — {reason}")]
    ComplianceDenied { guard: String, reason: String },

    #[error("compliance deferred: {reason}")]
    ComplianceDeferred { reason: String },

    #[error("account restricted: {reason}")]
    AccountRestricted { reason: String },

    #[error("RLS context not set — request must be authenticated")]
    RlsContextMissing,

    #[error("tenant mismatch: token claims {token_member} but DB session is {session_member}")]
    TenantMismatch { token_member: String, session_member: String },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("gRPC transport: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("gRPC status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("vault error: {0}")]
    Vault(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("proto decode error: {0}")]
    ProtoDecode(String),

    #[error("optimistic concurrency conflict (version mismatch)")]
    OptimisticConflict,

    #[error("upstream timeout: {0}")]
    Timeout(String),

    #[error("circuit breaker open for {service}")]
    CircuitOpen { service: String },

    #[error("restriction signal: {signal_kind} — {reason}")]
    RestrictionSignal { signal_kind: String, reason: String },

    #[error("internal: {0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result alias bound to [`LccError`].
pub type LccResult<T> = Result<T, LccError>;

impl LccError {
    /// The stable machine-readable code carried in the canonical error
    /// envelope. These strings are part of the public API contract, so clients
    /// switch on them; changing one is a breaking change.
    pub fn error_code(&self) -> &'static str {
        match self {
            LccError::Validation(_) => "validation_failed",
            LccError::NotFound { .. } => "not_found",
            LccError::Conflict(_) | LccError::OptimisticConflict => "conflict",
            LccError::Forbidden(_) => "forbidden",
            LccError::Unauthorized(_) => "unauthorized",
            LccError::PermitTokenInvalid(_) => "permit_token_invalid",
            LccError::RateLimited { .. } => "rate_limited",
            LccError::ComplianceDenied { .. } => "compliance_denied",
            LccError::ComplianceDeferred { .. } => "compliance_deferred",
            LccError::AccountRestricted { .. } | LccError::RestrictionSignal { .. } => {
                "account_restricted"
            }
            LccError::RlsContextMissing | LccError::TenantMismatch { .. } => "tenant_mismatch",
            LccError::CircuitOpen { .. } => "circuit_open",
            LccError::Timeout(_) => "upstream_timeout",
            LccError::Database(_) | LccError::Redis(_) | LccError::HttpClient(_) => {
                "dependency_error"
            }
            LccError::Config(_) => "config_error",
            LccError::Internal(_) | LccError::Other(_) => "internal_error",
            _ => "internal_error",
        }
    }

    /// Renders this error in the canonical envelope shape,
    /// `{ "error": { code, message, trace_id } }`.
    pub fn to_envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope::new(self.error_code(), self.to_string())
    }

    /// Returns the HTTP status code equivalent for this error.
    pub fn http_status(&self) -> http::StatusCode {
        use http::StatusCode;
        match self {
            LccError::Validation(_) => StatusCode::BAD_REQUEST,
            LccError::NotFound { .. } => StatusCode::NOT_FOUND,
            LccError::Conflict(_) | LccError::OptimisticConflict => StatusCode::CONFLICT,
            LccError::Forbidden(_) => StatusCode::FORBIDDEN,
            LccError::Unauthorized(_) | LccError::PermitTokenInvalid(_) => StatusCode::UNAUTHORIZED,
            LccError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            LccError::ComplianceDenied { .. } | LccError::ComplianceDeferred { .. } => {
                StatusCode::FORBIDDEN
            }
            LccError::AccountRestricted { .. } | LccError::RestrictionSignal { .. } => {
                StatusCode::FORBIDDEN
            }
            LccError::RlsContextMissing | LccError::TenantMismatch { .. } => StatusCode::UNAUTHORIZED,
            LccError::CircuitOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
            LccError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Maps this error to a [`tonic::Status`].
    pub fn to_grpc_status(&self) -> tonic::Status {
        use tonic::Code;
        let code = match self {
            LccError::Validation(_) => Code::InvalidArgument,
            LccError::NotFound { .. } => Code::NotFound,
            LccError::Conflict(_) | LccError::OptimisticConflict => Code::AlreadyExists,
            LccError::Forbidden(_) => Code::PermissionDenied,
            LccError::Unauthorized(_) | LccError::PermitTokenInvalid(_) => Code::Unauthenticated,
            LccError::RateLimited { .. } => Code::ResourceExhausted,
            LccError::ComplianceDenied { .. } => Code::PermissionDenied,
            LccError::ComplianceDeferred { .. } => Code::FailedPrecondition,
            LccError::AccountRestricted { .. } | LccError::RestrictionSignal { .. } => {
                Code::PermissionDenied
            }
            LccError::RlsContextMissing | LccError::TenantMismatch { .. } => Code::Unauthenticated,
            LccError::CircuitOpen { .. } => Code::Unavailable,
            LccError::Timeout(_) => Code::DeadlineExceeded,
            _ => Code::Internal,
        };
        tonic::Status::new(code, self.to_string())
    }

    /// True if this error should never be retried.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            LccError::NotFound { .. }
                | LccError::Conflict(_)
                | LccError::Forbidden(_)
                | LccError::Unauthorized(_)
                | LccError::PermitTokenInvalid(_)
                | LccError::ComplianceDenied { .. }
                | LccError::AccountRestricted { .. }
                | LccError::RestrictionSignal { .. }
                | LccError::RlsContextMissing
                | LccError::TenantMismatch { .. }
                | LccError::OptimisticConflict
        )
    }

    /// True if this error represents a downstream-circuit-break condition.
    pub fn is_circuit_break(&self) -> bool {
        matches!(self, LccError::CircuitOpen { .. })
    }

    /// Public-facing, redacted message — never includes tokens, passwords, or secrets.
    pub fn public_message(&self) -> String {
        match self {
            LccError::Database(_)
            | LccError::Redis(_)
            | LccError::GrpcTransport(_)
            | LccError::HttpClient(_)
            | LccError::Vault(_)
            | LccError::Config(_)
            | LccError::Internal(_)
            | LccError::Other(_) => "internal server error".to_string(),
            other => other.to_string(),
        }
    }
}

/// Plain error envelope returned over HTTP / gRPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub trace_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl ErrorEnvelope {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorBody {
                code: code.into(),
                message: message.into(),
                trace_id: None,
                details: None,
            },
        }
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.error.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.error.details = Some(details);
        self
    }
}

/// Every service can render the same envelope, so the shape is defined once
/// here rather than re-implemented (and drifting) per service.
impl From<&LccError> for ErrorEnvelope {
    fn from(e: &LccError) -> Self {
        e.to_envelope()
    }
}

impl From<LccError> for ErrorEnvelope {
    fn from(e: LccError) -> Self {
        e.to_envelope()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_status_mapping() {
        assert_eq!(
            LccError::Validation("x".into()).http_status(),
            http::StatusCode::BAD_REQUEST
        );
        assert_eq!(
            LccError::NotFound {
                resource_type: "x".into(),
                resource_id: "y".into()
            }
            .http_status(),
            http::StatusCode::NOT_FOUND
        );
        assert_eq!(
            LccError::RateLimited { retry_after_ms: 100 }.http_status(),
            http::StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            LccError::ComplianceDenied {
                guard: "daily_cap".into(),
                reason: "exceeded".into()
            }
            .http_status(),
            http::StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn terminal_classification() {
        assert!(LccError::Forbidden("x".into()).is_terminal());
        assert!(LccError::OptimisticConflict.is_terminal());
        assert!(LccError::AccountRestricted {
            reason: "x".into()
        }
        .is_terminal());
        assert!(!LccError::Timeout("x".into()).is_terminal());
    }

    #[test]
    fn public_message_redacts_internal() {
        let internal = LccError::Internal("SELECT * FROM member_account WHERE token='abc'".into());
        assert_eq!(internal.public_message(), "internal server error");
    }

    #[test]
    fn error_envelope_shape() {
        let env = ErrorEnvelope::new("validation", "bad input")
            .with_trace_id("trace_123")
            .with_details(serde_json::json!({"field": "x"}));
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["error"]["code"], "validation");
        assert_eq!(json["error"]["message"], "bad input");
        assert_eq!(json["error"]["trace_id"], "trace_123");
    }
}
