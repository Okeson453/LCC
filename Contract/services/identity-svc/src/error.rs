//! identity-svc error types.
//!
//! Implements the canonical contract error envelope:
//!   { "error": { "code": "...", "message": "...", "details": {...}, "trace_id": "..." } }

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("upstream: {0}")]
    Upstream(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl Error {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::BadRequest(_) => "VALIDATION_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Conflict(_) => "CONFLICT",
            Self::Upstream(_) => "UPSTREAM_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status(&self) -> axum::http::StatusCode {
        match self {
            Self::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            Self::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
            Self::Conflict(_) => axum::http::StatusCode::CONFLICT,
            Self::Upstream(_) => axum::http::StatusCode::BAD_GATEWAY,
            Self::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let body = serde_json::json!({
            "error": {
                "code": self.code(),
                "message": self.to_string(),
                "trace_id": trace_id,
            }
        });
        (self.status(), axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(format!("db error: {e}"))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Upstream(format!("http error: {e}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::BadRequest(format!("json error: {e}"))
    }
}
