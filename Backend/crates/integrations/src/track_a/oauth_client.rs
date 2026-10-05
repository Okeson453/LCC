//! LinkedIn OAuth 2.0 helpers (token exchange, refresh, revocation).

use crate::track_a::{TrackAError, LINKEDIN_AUTH_TOKEN_URL};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_in: Option<i64>,
    pub scope: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse(pub TokenResponse);

/// Exchange authorization code for access + refresh tokens.
pub async fn exchange_code(
    client: &reqwest::Client,
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse, TrackAError> {
    let resp = client
        .post(LINKEDIN_AUTH_TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await
        .map_err(|e| TrackAError::Transport(e.to_string()))?;

    match resp.status() {
        reqwest::StatusCode::OK => resp
            .json::<TokenResponse>()
            .await
            .map_err(|e| TrackAError::Json(e.to_string())),
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

/// Refresh an access token using a refresh token.
pub async fn refresh_token(
    client: &reqwest::Client,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse, TrackAError> {
    let resp = client
        .post(LINKEDIN_AUTH_TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await
        .map_err(|e| TrackAError::Transport(e.to_string()))?;

    match resp.status() {
        reqwest::StatusCode::OK => resp
            .json::<TokenResponse>()
            .await
            .map_err(|e| TrackAError::Json(e.to_string())),
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
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

/// Revoke a refresh token (LinkedIn doesn't expose a revoke endpoint for
/// the v2 API; this is a placeholder for future support).
pub async fn revoke_token(
    _client: &reqwest::Client,
    _token: &str,
    _client_id: &str,
    _client_secret: &str,
) -> Result<(), TrackAError> {
    // LinkedIn v2 doesn't expose a token revocation endpoint. We delete the
    // token from our own vault + emit audit event; LinkedIn's grant is
    // effectively revoked by deleting the integration.
    Ok(())
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_response_serialization() {
        let r = TokenResponse {
            access_token: "abc".into(),
            expires_in: 3600,
            refresh_token: Some("xyz".into()),
            refresh_token_expires_in: Some(31_536_000),
            scope: "r_liteprofile r_emailaddress".into(),
            token_type: "Bearer".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: TokenResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.access_token, "abc");
        assert_eq!(back.refresh_token.unwrap(), "xyz");
    }
}
