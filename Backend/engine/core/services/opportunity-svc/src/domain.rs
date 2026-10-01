//! Opportunity-svc domain: discover, qualify, applications, proposals.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Uuid,
    pub member_id: Uuid,
    pub company_id: Option<Uuid>,
    pub title: String,
    pub source: Source,
    pub status: OpportunityStatus,
    pub fit_score: Option<f64>,
    pub discovered_at: DateTime<Utc>,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub version: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Manual,
    Inbound,
    LinkedInSearch,
    JobBoard,
    Referral,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Inbound => "inbound",
            Self::LinkedInSearch => "linkedin_search",
            Self::JobBoard => "job_board",
            Self::Referral => "referral",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Discovered,
    Qualified,
    Contacted,
    Conversation,
    Applied,
    Interviewing,
    Offer,
    Rejected,
    Closed,
    Withdrawn,
}

impl OpportunityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Qualified => "qualified",
            Self::Contacted => "contacted",
            Self::Conversation => "conversation",
            Self::Applied => "applied",
            Self::Interviewing => "interviewing",
            Self::Offer => "offer",
            Self::Rejected => "rejected",
            Self::Closed => "closed",
            Self::Withdrawn => "withdrawn",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: Uuid,
    pub member_id: Uuid,
    pub opportunity_id: Uuid,
    pub status: OpportunityStatus,
    pub submitted_at: DateTime<Utc>,
    pub resume_doc_id: Option<Uuid>,
    pub cover_letter: Option<String>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Uuid,
    pub member_id: Uuid,
    pub opportunity_id: Uuid,
    pub title: String,
    pub body: String,
    pub price_cents: Option<i64>,
    pub currency: Option<String>,
    pub status: ProposalStatus,
    pub kb_ref_ids: Vec<Uuid>,
    pub sent_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Sent,
    Accepted,
    Rejected,
    Withdrawn,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Sent => "sent",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}
