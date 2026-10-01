//! Auth start handler — begins LinkedIn OAuth flow.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthStartQuery {
    pub member_id: String,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthStartResponse {
    pub auth_url: String,
    pub state: String,
    pub scopes: Vec<String>,
}

const SCOPES: &[&str] = &["r_liteprofile", "r_emailaddress", "w_member_social"];

pub async fn auth_start(
    State(state): State<AppState>,
    Query(q): Query<AuthStartQuery>,
) -> Json<AuthStartResponse> {
    let state_id = q.state.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let redirect = q
        .redirect_uri
        .unwrap_or_else(|| "https://api.lcc.example/auth/callback".to_string());

    let auth_url = format!(
        "https://www.linkedin.com/oauth/v2/authorization?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
        state.config().auth_jwt_issuer, // placeholder; in prod use the OAuth client_id
        urlencoding::encode(&redirect),
        SCOPES.join("+"),
        state_id,
    );
    Json(AuthStartResponse {
        auth_url,
        state: state_id,
        scopes: SCOPES.iter().map(|s| s.to_string()).collect(),
    })
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.replace(' ', "%20")
    }
}
