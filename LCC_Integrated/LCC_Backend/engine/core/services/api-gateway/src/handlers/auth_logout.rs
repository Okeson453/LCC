//! Logout handler.

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::state::AppState;

pub async fn auth_logout(State(_state): State<AppState>) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
