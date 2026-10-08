//! Opportunity-svc domain: discover, qualify, applications (job applications
//! and client proposals).
//!
//! # Where every field comes from
//!
//! The shapes below are reconciled against the *real* schema, not against the
//! names this service used to invent:
//!
//! * `lcc.opportunities` — created by `0009_opportunity_outreach.sql`, then
//!   brought up to the canonical contract shape by
//!   `0017_companies_and_interactions.sql` (which added `status`, `fit_score`,
//!   `company_id`, `source`, `discovered_at`, `last_evaluated_at`, `version`
//!   and explicitly declares that the 0009 table was the stale artifact).
//! * `lcc.applications` — created by `0014_application_and_message_template.sql`.
//!   Its own header comment reads: *"Applications (job-applications and
//!   client-proposals sent through the system). One row per
//!   (member_id, opportunity_id, application_type)."*
//!
//! # There is no `lcc.proposals` table, and there is not meant to be
//!
//! `lcc.proposals` does not exist in any migration and the *canonical contract*
//! never declares a `Proposal` entity either — only `ProposalDraft` (the body
//! returned by `POST …/draft-proposal`) and `ProposalSendRequest` (the body of
//! `POST …/send-proposal`), neither of which carries an `id`, a `status`, a
//! `version` or a `created_at` and so neither can be a persisted entity.
//! `POST …/send-proposal` responds with `components.schemas.Application`, and
//! `Contract/docs/endpoint_contract_matrix.md` names `lcc.applications` as the
//! backing table for *both* proposal endpoints. The design's own state table
//! says a sent proposal links an "`Application` entity".
//!
//! A client proposal is therefore an `lcc.applications` row with
//! `application_type = 'client_proposal'`. The old `Proposal` struct and the
//! old `lcc.proposals` table are gone; `Application` is the entity, and
//! `application_type` is the discriminator.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Opportunity
// ---------------------------------------------------------------------------

/// The real `lcc.opportunity_kind` ENUM (`0009`), verbatim.
///
/// **This is the opportunity CATEGORY, not the position.** The position an
/// opportunity represents is `lcc.opportunities.title` — see
/// [`Opportunity::position`]. Serving `kind` where a position is expected is
/// the specific defect this enum documents.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityKind {
    JobPosting,
    Consulting,
    Partnership,
    Speaking,
    Mentorship,
    Other,
}

impl OpportunityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::JobPosting => "job_posting",
            Self::Consulting => "consulting",
            Self::Partnership => "partnership",
            Self::Speaking => "speaking",
            Self::Mentorship => "mentorship",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "job_posting" => Self::JobPosting,
            "consulting" => Self::Consulting,
            "partnership" => Self::Partnership,
            "speaking" => Self::Speaking,
            "mentorship" => Self::Mentorship,
            "other" => Self::Other,
            _ => return None,
        })
    }
}

/// `Opportunity.type` from the canonical contract (`job | client`) — the
/// job-hunt vs. client-acquisition track split that Design §18.6 keys the
/// funnel on (`applied` (job) / `proposal_sent` (client)).
///
/// `lcc.opportunities` has no `type` column; the 6-value `kind` ENUM replaced
/// the contract's 2-value binary without the contract being updated. This is a
/// projection of `kind` onto the contract's vocabulary, **not** a new source of
/// truth, and the rule is deliberately conservative: only `job_posting` is the
/// job track. See `OPEN_QUESTIONS` in the service README — `speaking` and
/// `mentorship` are arguably neither, and the owner should decide.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityTrack {
    Job,
    Client,
}

impl OpportunityTrack {
    /// The job track is exactly the job-posting category; every other category
    /// is treated as client work, which is the pre-0009 binary the migration
    /// was written to generalise.
    pub fn from_kind(kind: OpportunityKind) -> Self {
        match kind {
            OpportunityKind::JobPosting => Self::Job,
            _ => Self::Client,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Client => "client",
        }
    }
}

/// The real `lcc.opportunity_funnel` ENUM, retained for completeness.
///
/// `0017` back-filled `status` from `funnel` and then declared `status` the
/// service-facing source of truth; `funnel` was left in place only so no other
/// consumer loses data. It is **not** written by this service, and there is no
/// total, information-preserving mapping between the two enums
/// (`status=interviewing`/`offer` has no `funnel` value, and
/// `funnel=proposed`/`negotiating` map to a `status` that depends on the
/// track). Inventing one would be a fabrication, so the column is left alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpportunityFunnel {
    Discovered,
    Qualified,
    InConversation,
    Proposed,
    Negotiating,
    Won,
    Lost,
}

impl OpportunityFunnel {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "discovered" => Self::Discovered,
            "qualified" => Self::Qualified,
            "in_conversation" => Self::InConversation,
            "proposed" => Self::Proposed,
            "negotiating" => Self::Negotiating,
            "won" => Self::Won,
            "lost" => Self::Lost,
            _ => return None,
        })
    }
}

/// `OpportunityStatus` from the canonical contract, verbatim:
/// `[discovered, qualified, contacted, conversation, applied, proposal_sent,
///  negotiation, won, closed_lost, discarded, dormant]`.
///
/// The previous enum in this file (`…interviewing, offer, rejected, closed,
/// withdrawn`) was a third variant found nowhere in the contract, the design or
/// the database; it was the stale artifact. `lcc.opportunities.status` is a
/// plain `TEXT` column with a partial index over the first four of these
/// values (`0017`), which is the decisive evidence for this list.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Discovered,
    Qualified,
    Contacted,
    Conversation,
    Applied,
    ProposalSent,
    Negotiation,
    Won,
    ClosedLost,
    Discarded,
    Dormant,
}

impl OpportunityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Qualified => "qualified",
            Self::Contacted => "contacted",
            Self::Conversation => "conversation",
            Self::Applied => "applied",
            Self::ProposalSent => "proposal_sent",
            Self::Negotiation => "negotiation",
            Self::Won => "won",
            Self::ClosedLost => "closed_lost",
            Self::Discarded => "discarded",
            Self::Dormant => "dormant",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "discovered" => Self::Discovered,
            "qualified" => Self::Qualified,
            "contacted" => Self::Contacted,
            "conversation" => Self::Conversation,
            "applied" => Self::Applied,
            "proposal_sent" => Self::ProposalSent,
            "negotiation" => Self::Negotiation,
            "won" => Self::Won,
            "closed_lost" => Self::ClosedLost,
            "discarded" => Self::Discarded,
            "dormant" => Self::Dormant,
            _ => return None,
        })
    }

    /// Terminal states: no further transition is meaningful.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Won | Self::ClosedLost | Self::Discarded)
    }
}

/// Where an opportunity was discovered from. `lcc.opportunities.source` is
/// `TEXT NOT NULL DEFAULT 'manual'` with no CHECK constraint, and the contract
/// types `Opportunity.source` as a bare `string`, so this list is the
/// service's own vocabulary rather than a database enum.
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

impl Source {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "manual" => Self::Manual,
            "inbound" => Self::Inbound,
            "linkedin_search" => Self::LinkedInSearch,
            "job_board" => Self::JobBoard,
            "referral" => Self::Referral,
            _ => return None,
        })
    }
}

/// One `lcc.opportunities` row, projected onto the canonical contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Uuid,
    pub member_id: Uuid,
    /// The opportunity CATEGORY (`lcc.opportunity_kind`). Never the position.
    pub kind: OpportunityKind,
    /// Contract `Opportunity.type`, derived from `kind` — see
    /// [`OpportunityTrack::from_kind`].
    #[serde(rename = "type")]
    pub track: OpportunityTrack,
    pub company_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    /// `lcc.opportunities.title` — the headline of the opportunity.
    pub title: String,
    /// The **position** this opportunity represents.
    ///
    /// Served from `lcc.opportunities.title`, which is the column that
    /// actually holds it ("Senior Backend Engineer — Acme"). `kind` is the
    /// category (`job_posting`) and is emphatically *not* the position. The
    /// contract requires `position` on
    /// `ApplicationSubmitRequest` and Design §11.8 puts `position` on the
    /// application row; since `lcc.applications` has no `position` column it
    /// is derived by joining back to the opportunity's `title`.
    pub position: String,
    pub company: Option<String>,
    pub source: Source,
    pub status: OpportunityStatus,
    pub fit_score: Option<f64>,
    /// Contract `fit_components` — the φ breakdown. Stored in
    /// `lcc.opportunities.phi_components` (0009), which is the same concept
    /// under its Greek-letter name.
    pub fit_components: JsonValue,
    pub discovered_at: DateTime<Utc>,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: JsonValue,
    pub version: i32,
}

// ---------------------------------------------------------------------------
// Applications (job applications AND client proposals)
// ---------------------------------------------------------------------------

/// `lcc.applications.application_type` — the real CHECK constraint from 0014,
/// and the same enum the contract's `Application.application_type` declares.
///
/// `ClientProposal` is the discriminator that replaces the invented
/// `lcc.proposals` table.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationType {
    JobApplication,
    ClientProposal,
    ExecutiveOutreach,
}

impl ApplicationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::JobApplication => "job_application",
            Self::ClientProposal => "client_proposal",
            Self::ExecutiveOutreach => "executive_outreach",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "job_application" => Self::JobApplication,
            "client_proposal" => Self::ClientProposal,
            "executive_outreach" => Self::ExecutiveOutreach,
            _ => return None,
        })
    }
}

/// `lcc.applications.status` — the real CHECK constraint from 0014 and the
/// contract's `Application.status` enum, which agree exactly.
///
/// This is deliberately NOT [`OpportunityStatus`]: the previous code wrote
/// `OpportunityStatus::Applied` (`"applied"`) into this column, which the
/// CHECK constraint rejects at runtime.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Draft,
    Queued,
    Sent,
    Replied,
    Rejected,
    Withdrawn,
}

impl ApplicationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::Replied => "replied",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "draft" => Self::Draft,
            "queued" => Self::Queued,
            "sent" => Self::Sent,
            "replied" => Self::Replied,
            "rejected" => Self::Rejected,
            "withdrawn" => Self::Withdrawn,
            _ => return None,
        })
    }
}

/// One `lcc.applications` row.
///
/// This is the entity for BOTH a job application and a client proposal; see
/// the module docs for the evidence that `lcc.proposals` was never real.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: Uuid,
    pub member_id: Uuid,
    pub opportunity_id: Uuid,
    pub application_type: ApplicationType,
    pub status: ApplicationStatus,
    /// The position, joined from `lcc.opportunities.title`. `None` only when
    /// the opportunity row has been deleted — `lcc.applications.opportunity_id`
    /// carries no foreign key, so this is reachable and is surfaced honestly
    /// rather than being defaulted to a placeholder.
    pub position: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub response_received_at: Option<DateTime<Utc>>,
    /// `lcc.applications.payload` — the variable-shape part of the submission.
    /// Carries the cover letter / resume reference for a job application and
    /// the drafted proposal body for a client proposal.
    pub payload: JsonValue,
    pub idempotency_key: Option<String>,
    /// `lcc.applications.version` is `BIGINT` (0014), not `INT`.
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The JSON body persisted in `lcc.applications.payload` for a client proposal.
///
/// `title` and `kb_refs` are contract-shaped (`ProposalDraft.title`,
/// `ProposalDraft.kb_refs`); `body` is the markdown of
/// `ProposalSendRequest.body_markdown`.
///
/// `price_cents` / `currency` carry a **flagged** caveat: nothing in the
/// canonical contract or in the design defines a pricing model for a proposal
/// (`ProposalSendRequest` has no money field; §3.6 of the design framework
/// describes a one-pager grounded in the KB, not a priced engagement). They are
/// kept as opaque caller-supplied data in `payload` so the existing API does not
/// silently discard input, but nothing in this service interprets them. See
/// `OPEN_QUESTIONS` in the service README.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalPayload {
    pub title: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kb_refs: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl ProposalPayload {
    pub fn to_json(&self) -> JsonValue {
        json!({
            "title": self.title,
            "body": self.body,
            "kb_refs": self.kb_refs,
            "price_cents": self.price_cents,
            "currency": self.currency,
        })
    }
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/// `components.schemas.OpportunityPage` — `{items, next_cursor, has_more}`.
///
/// Replaces the bare JSON array this service used to return, which the contract
/// does not declare.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, has_more: bool, next_cursor: Option<String>) -> Self {
        Self {
            items,
            next_cursor,
            has_more,
        }
    }
}

/// Opaque keyset cursor over `(fit_score DESC NULLS LAST, discovered_at DESC,
/// id DESC)`.
///
/// `fit_score` is `NULL` for every opportunity that has not been scored, and
/// the sort is `NULLS LAST`, so a naive `(fit_score, …) < (cursor, …)` row
/// comparison is wrong at the NULL boundary. [`Page`] therefore carries the
/// scored/unscored split explicitly: `fit: None` means "the cursor is inside the
/// unscored block, keep walking it".
#[derive(Debug, Clone, PartialEq)]
pub struct OpportunityCursor {
    pub fit: Option<f64>,
    pub discovered_at: DateTime<Utc>,
    pub id: Uuid,
}

impl OpportunityCursor {
    /// `|` is a legal sub-delim in a query string, chrono serialises `Utc` with
    /// a `Z` suffix (never a `+`, which would decode as a space), and UUIDs are
    /// hex. So this needs no percent-encoding round-trip.
    pub fn encode(&self) -> String {
        let fit = match self.fit {
            Some(f) => f.to_string(),
            None => String::new(),
        };
        format!(
            "{fit}|{}|{}",
            self.discovered_at
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            self.id
        )
    }

    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split('|');
        let fit = parts.next()?;
        let discovered_at = DateTime::parse_from_rfc3339(parts.next()?).ok()?;
        let id = Uuid::parse_str(parts.next()?).ok()?;
        if parts.next().is_some() {
            return None;
        }
        let fit = if fit.is_empty() {
            None
        } else {
            let v: f64 = fit.parse().ok()?;
            if !v.is_finite() {
                return None;
            }
            Some(v)
        };
        Some(Self {
            fit,
            discovered_at: discovered_at.with_timezone(&Utc),
            id,
        })
    }
}
