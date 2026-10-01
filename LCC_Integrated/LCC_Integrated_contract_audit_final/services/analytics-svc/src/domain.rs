//! Analytics domain.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub member_id: Uuid,
    pub as_of: DateTime<Utc>,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub content_metrics: ContentMetrics,
    pub outreach_metrics: OutreachMetrics,
    pub engagement_metrics: EngagementMetrics,
    pub opportunity_metrics: OpportunityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetrics {
    pub published: i64,
    pub scheduled: i64,
    pub draft: i64,
    pub in_review: i64,
    pub average_quality_loop_count: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutreachMetrics {
    pub active_sequences: i64,
    pub paused_sequences: i64,
    pub step_replies: i64,
    pub reply_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub inbound_count: i64,
    pub queued_tasks: i64,
    pub completed_tasks: i64,
    pub draft_ready_tasks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityMetrics {
    pub discovered: i64,
    pub qualified: i64,
    pub applied: i64,
    pub interviewing: i64,
    pub offers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: NaiveDate,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub metric: String,
    pub granularity: Granularity,
    pub points: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Granularity {
    Day,
    Week,
    Month,
}

impl Granularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }
}
