//! Orchestrator domain — the daily briefing and its input sections.
//!
//! The briefing aggregates rows from four different services' tables. Each
//! source gets its own submodule (mirroring the owning service) so the
//! projection types stay next to the shape they project, and the aggregate
//! `Briefing` composes them.

pub mod lcc_approval;
pub mod lcc_engagement;
pub mod lcc_opportunity;
pub mod lcc_outreach;

use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::briefing_kind_enum::BriefingKind;

use lcc_approval::ApprovalSummary;
use lcc_engagement::TaskSummary;
use lcc_opportunity::OpportunitySummary;
use lcc_outreach::SequenceSummary;

/// The four sections assembled into a briefing, each already limited and
/// ordered by `PgRepository::assemble_briefing`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingSections {
    /// Pending approvals, oldest first — these block every downstream action.
    pub approvals_due: Vec<ApprovalSummary>,
    /// Opportunities at or above the φ hot threshold.
    pub hot_opportunities: Vec<OpportunitySummary>,
    /// Engagement tasks queued or drafted.
    pub engagement: Vec<TaskSummary>,
    /// Active sequences with a step that is due now.
    pub followups: Vec<SequenceSummary>,
}

/// A member's daily briefing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Briefing {
    pub member_id: Uuid,
    pub date: NaiveDate,
    pub generated_at: DateTime<Utc>,
    pub kind: BriefingKind,
    pub sections: BriefingSections,
}
