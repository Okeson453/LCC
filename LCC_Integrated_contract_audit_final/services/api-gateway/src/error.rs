//! api-gateway error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiGatewayError {
    #[error("invalid request: {0}")]
    BadRequest(String),

    #[error("upstream error: {0}")]
    Upstream(String),

    /// No upstream is bound for the requested domain, or the upstream has no
    /// route for the forwarded path. Surfaced as a 404 naming the missing
    /// binding rather than being masked as a success.
    #[error("not found: {0}")]
    NotFound(String),

    #[error("auth failed: {0}")]
    Unauthorized(String),

    #[error("rate limited")]
    RateLimited,

    #[error("internal: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiGatewayError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, code) = match &self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::Upstream(_) => (StatusCode::BAD_GATEWAY, "upstream_error"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let body = serde_json::json!({
            "error": {
                "code": code,
                "message": self.to_string(),
            }
        });
        (status, axum::Json(body)).into_response()
    }
}

impl From<reqwest::Error> for ApiGatewayError {
    fn from(e: reqwest::Error) -> Self {
        Self::Upstream(e.to_string())
    }
}
