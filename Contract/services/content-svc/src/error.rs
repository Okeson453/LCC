//! Content-svc error envelope.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("validation: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("governance denied: {0}")]
    GovernanceDenied(String),

    #[error("state transition: {from} -> {to} is not allowed")]
    InvalidTransition { from: String, to: String },

    #[error("quality check failed after {loop_count} iterations")]
    QualityLoopExhausted { loop_count: i32 },

    #[error("upstream: {0}")]
    Upstream(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::Conflict(_) => "CONFLICT",
            Self::GovernanceDenied(_) => "GOVERNANCE_DENIED",
            Self::InvalidTransition { .. } => "INVALID_STATE_TRANSITION",
            Self::QualityLoopExhausted { .. } => "QUALITY_LOOP_EXHAUSTED",
            Self::Upstream(_) => "UPSTREAM_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
    fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden | Self::GovernanceDenied(_) | Self::QualityLoopExhausted { .. } => {
                StatusCode::FORBIDDEN
            }
            Self::Conflict(_) | Self::InvalidTransition { .. } => StatusCode::CONFLICT,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let body = serde_json::json!({
            "error": { "code": self.code(), "message": self.to_string(), "trace_id": trace_id }
        });
        (self.status(), axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(format!("db: {e}"))
    }
}
