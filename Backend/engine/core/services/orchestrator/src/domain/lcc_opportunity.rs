//! Opportunity rows projected into the briefing ("what is worth acting on").

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An opportunity above the φ hot threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunitySummary {
    pub id: Uuid,
    /// `lcc.companies.name`, absent when the opportunity has no company.
    pub company_name: Option<String>,
    pub position: Option<String>,
    /// φ fit score; the briefing only includes rows at or above the hot cutoff.
    pub fit_score: Option<f64>,
    pub status: String,
    pub discovered_at: DateTime<Utc>,
}
