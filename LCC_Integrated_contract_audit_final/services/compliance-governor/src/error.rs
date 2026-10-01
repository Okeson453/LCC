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
