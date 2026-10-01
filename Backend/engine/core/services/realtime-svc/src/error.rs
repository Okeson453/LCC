//! Error type for the realtime service.

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

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, code) = match &self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            Self::Upstream(_) => (StatusCode::BAD_GATEWAY, "UPSTREAM_ERROR"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };
        let trace_id = uuid::Uuid::new_v4().to_string();
        let body = serde_json::json!({
            "error": { "code": code, "message": self.to_string(), "trace_id": trace_id }
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(format!("db: {e}"))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Upstream(format!("http: {e}"))
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
