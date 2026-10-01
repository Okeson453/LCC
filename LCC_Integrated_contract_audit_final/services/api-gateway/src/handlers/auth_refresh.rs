//! OAuth refresh handler.

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct AuthRefreshResponse {
    pub member_id: String,
    pub new_expires_at: String,
    pub refreshed_at: String,
}

pub async fn auth_refresh(State(_state): State<AppState>) -> Json<AuthRefreshResponse> {
    Json(AuthRefreshResponse {
        member_id: "00000000-0000-0000-0000-000000000001".into(),
        new_expires_at: "2025-12-31T00:00:00Z".into(),
        refreshed_at: chrono::Utc::now().to_rfc3339(),
    })
}
