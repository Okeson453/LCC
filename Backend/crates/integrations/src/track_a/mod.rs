//! Track A — official LinkedIn API clients.
//!
//! These wrap the LinkedIn Marketing/Share API, OIDC userinfo, and Jobs
//! endpoints. The integration-gateway is the only caller.

pub mod jobs_client;
pub mod oauth_client;
pub mod profile_client;
pub mod share_client;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrackAError {
    #[error("linkedin api error (status={status}): {body}")]
    Api { status: u16, body: String },
    #[error("linkedin auth error: {0}")]
    Auth(String),
    #[error("linkedin rate limit (429): retry_after_ms={0}")]
    RateLimited(u64),
    #[error("linkedin restriction signal: {kind} — {body}")]
    Restriction { kind: String, body: String },
    #[error("transport: {0}")]
    Transport(String),
    #[error("json: {0}")]
    Json(String),
    #[error("config: {0}")]
    Config(String),
}

/// LinkedIn API base URLs.
pub const LINKEDIN_API_BASE: &str = "https://api.linkedin.com";
pub const LINKEDIN_REST_BASE: &str = "https://rest.api.linkedin.com";
pub const LINKEDIN_AUTH_BASE: &str = "https://www.linkedin.com/oauth";
pub const LINKEDIN_AUTH_TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
