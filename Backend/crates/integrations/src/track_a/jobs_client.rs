//! LinkedIn Jobs API client (Track A, read-only).

use crate::track_a::{TrackAError, LINKEDIN_REST_BASE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListing {
    pub id: String,
    pub title: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub listed_at: Option<String>,
    pub apply_url: Option<String>,
    pub salary_range: Option<SalaryRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSearchResponse {
    pub elements: Vec<JobListing>,
    pub paging: Option<Paging>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub count: u32,
    pub start: u32,
    pub total: Option<u32>,
}

/// Search jobs. Subject to the documented rate limits — Source §38.
pub async fn search_jobs(
    client: &reqwest::Client,
    access_token: &str,
    keywords: &str,
    location: Option<&str>,
    start: u32,
    count: u32,
) -> Result<JobSearchResponse, TrackAError> {
    let mut url = format!(
        "{}/rest/jobSearch?q=keywords&keywords={}&start={}&count={}",
        LINKEDIN_REST_BASE,
        urlencoding::encode(keywords),
        start,
        count.min(50) // hard cap
    );
    if let Some(loc) = location {
        url.push_str(&format!("&location={}", urlencoding::encode(loc)));
    }

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| TrackAError::Transport(e.to_string()))?;

    match resp.status() {
        reqwest::StatusCode::OK => resp
            .json::<JobSearchResponse>()
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

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.bytes()
            .flat_map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    vec![b as char].into_iter().collect::<Vec<_>>().into_iter()
                }
                other => vec![format!("%{:02X}", other).chars().collect::<Vec<_>>()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into_iter(),
            })
            .collect()
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_listing_roundtrip() {
        let j = JobListing {
            id: "1234".into(),
            title: "Senior Engineer".into(),
            company: Some("Acme".into()),
            location: Some("Remote".into()),
            description: None,
            listed_at: Some("2026-04-15".into()),
            apply_url: Some("https://linkedin.com/jobs/1234".into()),
            salary_range: Some(SalaryRange {
                min: Some(150_000),
                max: Some(200_000),
                currency: Some("USD".into()),
            }),
        };
        let s = serde_json::to_string(&j).unwrap();
        let back: JobListing = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, "1234");
    }
}
