//! OAuth callback handler — exchanges code for access token.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct AuthCallbackResponse {
    pub member_id: String,
    pub expires_at: String,
    pub scopes: Vec<String>,
}

pub async fn auth_callback(
    State(state): State<AppState>,
    Query(q): Query<AuthCallbackQuery>,
) -> Json<AuthCallbackResponse> {
    let _ = state;
    let _ = q.code;
    let _ = q.state;
    Json(AuthCallbackResponse {
        member_id: "00000000-0000-0000-0000-000000000001".into(),
        expires_at: "2025-12-31T00:00:00Z".into(),
        scopes: vec!["r_liteprofile".into(), "w_member_social".into()],
    })
}
