//! Error type for the Compliance Governor service.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GovernorError {
    #[error("lcc error: {0}")]
    Lcc(#[from] lcc_error::LccError),
    #[error("config error: {0}")]
    Config(String),
    #[error("compliance config error: {0}")]
    ComplianceConfig(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tonic transport: {0}")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("two-reviewer sign-off required: only {0} of 2 reviewers")]
    TwoReviewerRequired(usize),
    #[error("compliance config activate failed: {0}")]
    ActivateFailed(String),
    /// No row matched — e.g. activating a config-version id that does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// Catch-all for failures that have no more specific variant — currently
    /// the sqlx errors raised by the config-version persistence paths.
    #[error("internal error: {0}")]
    Internal(String),
}

impl GovernorError {
    /// Canonical HTTP status for this error, used by the REST adapters.
    pub fn status_code(&self) -> u16 {
        match self {
            GovernorError::NotFound(_) => 404,
            GovernorError::TwoReviewerRequired(_) => 422,
            GovernorError::Config(_)
            | GovernorError::ComplianceConfig(_)
            | GovernorError::ActivateFailed(_) => 400,
            _ => 500,
        }
    }
}

/// Renders the canonical error envelope the contract specifies
/// (`{ error: { code, message } }`) so the governor's REST routes return the
/// same shape as the rest of the platform.
impl axum::response::IntoResponse for GovernorError {
    fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse as _;
        let status = axum::http::StatusCode::from_u16(self.status_code())
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        let code = match self {
            GovernorError::NotFound(_) => "not_found",
            GovernorError::TwoReviewerRequired(_) => "two_reviewer_required",
            GovernorError::Config(_) | GovernorError::ComplianceConfig(_) => "invalid_config",
            GovernorError::ActivateFailed(_) => "activation_failed",
            _ => "internal_error",
        };
        let body = serde_json::json!({
            "error": { "code": code, "message": self.to_string() }
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<lcc_compliance::config::ComplianceConfigError> for GovernorError {
    fn from(e: lcc_compliance::config::ComplianceConfigError) -> Self {
        GovernorError::ComplianceConfig(e.to_string())
    }
}

impl From<lcc_compliance::permit_token::PermitError> for GovernorError {
    fn from(e: lcc_compliance::permit_token::PermitError) -> Self {
        GovernorError::ActivateFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_display() {
        let e = GovernorError::Config("bad".into());
        assert!(e.to_string().contains("config"));
    }
}
