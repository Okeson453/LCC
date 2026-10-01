//! Orchestrator error envelope.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("upstream: {0}")]
    Upstream(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl Error {
    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Upstream(_) => "UPSTREAM_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
    fn status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::BadRequest(format!("json: {e}"))
    }
}

impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Self::Upstream(format!("redis: {e}"))
    }
}
