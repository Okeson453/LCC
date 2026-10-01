//! LinkedIn profile client — `/v2/userinfo` (OIDC).
//!
//! Read-only. Used by identity-svc at refresh to verify the OAuth grant is
//! still active (Source §18).

use crate::track_a::{TrackAError, LINKEDIN_API_BASE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,                 // LinkedIn member ID
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub locale: Option<String>,
    pub picture: Option<String>,
}

/// Fetch userinfo via the OIDC userinfo endpoint. Bearer token is the LCC
/// member's access token.
pub async fn fetch_userinfo(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<UserInfo, TrackAError> {
    let resp = client
        .get(format!("{}/v2/userinfo", LINKEDIN_API_BASE))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| TrackAError::Transport(e.to_string()))?;

    match resp.status() {
        reqwest::StatusCode::OK => resp
            .json::<UserInfo>()
            .await
            .map_err(|e| TrackAError::Json(e.to_string())),
        reqwest::StatusCode::UNAUTHORIZED => {
            let body = resp.text().await.unwrap_or_default();
            Err(TrackAError::Auth(body))
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => Err(TrackAError::RateLimited(60_000)),
        s => {
            let body = resp.text().await.unwrap_or_default();
            Err(TrackAError::Api {
                status: s.as_u16(),
                body,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn userinfo_roundtrip() {
        let u = UserInfo {
            sub: "abc123".into(),
            name: Some("Alice".into()),
            given_name: Some("Alice".into()),
            family_name: Some("Doe".into()),
            email: Some("a@example.com".into()),
            email_verified: Some(true),
            locale: Some("en_US".into()),
            picture: None,
        };
        let s = serde_json::to_string(&u).unwrap();
        let back: UserInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(back.sub, "abc123");
    }
}
