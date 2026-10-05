//! LinkedIn Share / UGC Post client (Track A).
//!
//! Per the spec, this is the *only* LinkedIn endpoint where personal-profile
//! auto-publishing is supported by the official API. Other personal-profile
//! actions (DMs, connection requests, etc.) are NOT supported and must route
//! to Track B (human-assist).

use crate::track_a::{TrackAError, LINKEDIN_REST_BASE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcPostRequest {
    pub author_urn: String, // "urn:li:person:{id}" or "urn:li:organization:{id}"
    pub commentary: String,
    pub visibility: String, // "PUBLIC" | "CONNECTIONS"
    pub distribution: Option<Distribution>,
    pub media: Option<Vec<Media>>,
    pub lifecycle_state: String, // "PUBLISHED"
    pub is_api_call: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub feed_distribution: String, // "MAIN_FEED" | "NONE"
    pub target_entities: Vec<String>,
    pub third_party_distribution_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub category: String, // "IMAGE" | "VIDEO" | "ARTICLE"
    pub media_urn: String,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcPostResponse {
    pub id: String, // "urn:li:ugcPost:{id}"
    pub created_at: i64,
}

/// Publish a UGC post. The caller is responsible for holding a valid
/// `permit_token` and the action already passed the Compliance Governor.
pub async fn publish_ugc_post(
    client: &reqwest::Client,
    access_token: &str,
    request: &UgcPostRequest,
    idempotency_key: &str,
) -> Result<UgcPostResponse, TrackAError> {
    let url = format!("{}/rest/posts", LINKEDIN_REST_BASE);
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .header("X-Restli-Protocol-Version", "2.0.0")
        .header("Idempotency-Key", idempotency_key)
        .json(request)
        .send()
        .await
        .map_err(|e| TrackAError::Transport(e.to_string()))?;

    match resp.status() {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => resp
            .json::<UgcPostResponse>()
            .await
            .map_err(|e| TrackAError::Json(e.to_string())),
        reqwest::StatusCode::TOO_MANY_REQUESTS => Err(TrackAError::RateLimited(60_000)),
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            let body = resp.text().await.unwrap_or_default();
            // Detect restriction signal in body.
            if body.contains("restricted") || body.contains("verification") {
                Err(TrackAError::Restriction {
                    kind: "account_restricted".into(),
                    body,
                })
            } else {
                Err(TrackAError::Auth(body))
            }
        }
        s => {
            let body = resp.text().await.unwrap_or_default();
            Err(TrackAError::Api {
                status: s.as_u16(),
                body,
            })
        }
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ugc_post_serialization() {
        let req = UgcPostRequest {
            author_urn: "urn:li:person:abc".into(),
            commentary: "Hello world!".into(),
            visibility: "PUBLIC".into(),
            distribution: Some(Distribution {
                feed_distribution: "MAIN_FEED".into(),
                target_entities: vec![],
                third_party_distribution_tags: vec![],
            }),
            media: None,
            lifecycle_state: "PUBLISHED".into(),
            is_api_call: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("PUBLISHED"));
        assert!(json.contains("urn:li:person:abc"));
    }
}
