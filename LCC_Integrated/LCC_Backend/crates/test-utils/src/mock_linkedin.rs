//! Mock LinkedIn server (wiremock-backed).
//!
//! Provides a fake of the supported LinkedIn API surface for integration tests.
//! Configurable per-endpoint: 200 / 429 / 200+restriction-banner / 401.

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct MockLinkedin {
    pub server: MockServer,
}

impl MockLinkedin {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub fn base_url(&self) -> String {
        self.server.uri()
    }

    /// Default successful userinfo.
    pub async fn mock_userinfo_ok(&self) {
        Mock::given(method("GET"))
            .and(path("/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "sub": "li_abc",
                "name": "Test User",
                "email": "test@example.com",
            })))
            .mount(&self.server)
            .await;
    }

    /// Always-429 userinfo (rate limit simulation).
    pub async fn mock_userinfo_429(&self) {
        Mock::given(method("GET"))
            .and(path("/v2/userinfo"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "60"))
            .mount(&self.server)
            .await;
    }

    /// 200 with restriction-banner-like content (triggers restriction signal).
    pub async fn mock_userinfo_restricted(&self) {
        Mock::given(method("GET"))
            .and(path("/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "Your account has been restricted. Please verify your identity.".to_string(),
            ))
            .mount(&self.server)
            .await;
    }

    /// Default successful UGC post publish.
    pub async fn mock_ugc_post_ok(&self) {
        Mock::given(method("POST"))
            .and(path("/rest/posts"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "urn:li:ugcPost:12345",
                "createdAt": 1714694400000i64,
            })))
            .mount(&self.server)
            .await;
    }

    /// 401 (token revoked).
    pub async fn mock_401(&self) {
        Mock::given(method("POST"))
            .and(path("/rest/posts"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&self.server)
            .await;
    }

    /// 429 for posts.
    pub async fn mock_429_posts(&self) {
        Mock::given(method("POST"))
            .and(path("/rest/posts"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "30"))
            .mount(&self.server)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_starts() {
        let m = MockLinkedin::start().await;
        assert!(m.base_url().starts_with("http://"));
    }
}
