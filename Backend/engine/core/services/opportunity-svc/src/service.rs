//! Opportunity service.
//!
//! Owns the business rules; the repository owns the SQL. Everything here is
//! member-scoped end to end: the `member_id` that reaches this layer is the
//! JWT subject, and it is threaded into every query as the tenant predicate.

use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::domain::{
    Application, ApplicationStatus, ApplicationType, Opportunity, OpportunityCursor,
    OpportunityKind, OpportunityStatus, Page, ProposalPayload, Source,
};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    #[allow(dead_code)]
    redis: deadpool_redis::Pool,
}

/// Bounds on the `limit` query parameter, matching the canonical contract's
/// `Limit` parameter (`minimum: 1, maximum: 100`).
pub const MIN_LIMIT: i64 = 1;
pub const MAX_LIMIT: i64 = 100;
pub const DEFAULT_LIMIT: i64 = 25;

/// Clamp a caller-supplied limit into the contract's range. Out-of-range values
/// are clamped rather than rejected so an over-eager client gets a page, not a
/// 400; a non-positive value is clamped up to `MIN_LIMIT` so a `LIMIT 0` can
/// never masquerade as "no results".
pub fn clamp_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(MIN_LIMIT, MAX_LIMIT)
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }

    pub async fn list(
        &self,
        m: Uuid,
        s: Option<OpportunityStatus>,
        limit: i64,
        cursor: Option<OpportunityCursor>,
    ) -> Result<Page<Opportunity>, Error> {
        self.repo.list(m, s, limit, cursor).await
    }

    pub async fn get(&self, m: Uuid, id: Uuid) -> Result<Opportunity, Error> {
        self.repo.get(m, id).await
    }

    /// Record a new opportunity.
    ///
    /// `kind` is the opportunity CATEGORY. When the caller does not supply one,
    /// `opportunity_kind.other` is stored — that value exists in the schema
    /// precisely to mean "no more specific category", so defaulting to it does
    /// not assert something the caller did not say. Guessing `job_posting`
    /// instead would silently mis-classify every client engagement.
    #[allow(clippy::too_many_arguments)]
    pub async fn discover(
        &self,
        member_id: Uuid,
        title: &str,
        kind: Option<OpportunityKind>,
        source: Source,
        company_id: Option<Uuid>,
        contact_id: Option<Uuid>,
        company: Option<String>,
        metadata: JsonValue,
    ) -> Result<Opportunity, Error> {
        if title.trim().is_empty() {
            return Err(Error::Validation("title required".into()));
        }
        if company_id.is_some() && contact_id.is_some() {
            // `lcc.opportunities` has independent FKs to `lcc.companies` and
            // `lcc.contacts`; nothing stops a company and a contact from
            // belonging to different members, and a cross-tenant contact on a
            // member's own opportunity is a leak waiting to happen.
            self.repo
                .assert_references_owned(member_id, company_id, contact_id)
                .await?;
        }
        let now = Utc::now();
        let o = Opportunity {
            id: Uuid::new_v4(),
            member_id,
            kind: kind.unwrap_or(OpportunityKind::Other),
            track: crate::domain::OpportunityTrack::from_kind(
                kind.unwrap_or(OpportunityKind::Other),
            ),
            company_id,
            contact_id,
            position: title.into(),
            title: title.into(),
            company,
            source,
            status: OpportunityStatus::Discovered,
            fit_score: None,
            fit_components: json!({}),
            discovered_at: now,
            last_evaluated_at: None,
            created_at: now,
            updated_at: now,
            metadata: if metadata.is_null() {
                json!({})
            } else {
                metadata
            },
            version: 1,
        };
        self.repo.insert(&o).await?;
        Ok(o)
    }

    /// Recompute φ and move the opportunity to `status`.
    pub async fn qualify(
        &self,
        m: Uuid,
        id: Uuid,
        v: i32,
        fit_score: f64,
        next: OpportunityStatus,
        fit_components: JsonValue,
    ) -> Result<Opportunity, Error> {
        if !(0.0..=1.0).contains(&fit_score) {
            return Err(Error::Validation("fit_score must be 0..=1".into()));
        }
        let new_v = self
            .repo
            .qualify(m, id, v, fit_score, next, &fit_components)
            .await?;
        // `get` is member-scoped, so a version conflict raised against another
        // member's id surfaces here as NotFound rather than as a success.
        let mut o = self.repo.get(m, id).await?;
        o.version = new_v;
        Ok(o)
    }

    /// Draft a job application against an opportunity.
    ///
    /// The row is persisted as `status = 'draft'` with `submitted_at = NULL`:
    /// actually submitting is Tier-5 and gated behind a human approval that
    /// this service does not own, so claiming `sent` here would be a false 200.
    pub async fn apply(
        &self,
        member_id: Uuid,
        opportunity_id: Uuid,
        cover_letter: Option<String>,
        resume_doc_id: Option<Uuid>,
    ) -> Result<Application, Error> {
        // `lcc.applications.opportunity_id` has no foreign key, so the
        // opportunity is resolved (and tenant-checked) before the write. A
        // missing or foreign opportunity is a 404, not an orphan row.
        let opp = self.repo.get(member_id, opportunity_id).await?;
        let now = Utc::now();
        let a = Application {
            id: Uuid::new_v4(),
            member_id,
            opportunity_id,
            application_type: ApplicationType::JobApplication,
            status: ApplicationStatus::Draft,
            position: Some(opp.title.clone()),
            submitted_at: None,
            response_received_at: None,
            payload: json!({
                "position": opp.title,
                "cover_letter": cover_letter,
                "resume_doc_id": resume_doc_id,
            }),
            idempotency_key: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_application(&a).await?;
        Ok(a)
    }

    pub async fn list_applications(&self, m: Uuid, limit: i64) -> Result<Vec<Application>, Error> {
        self.repo.list_applications(m, None, limit).await
    }

    /// Draft a client proposal against an opportunity.
    ///
    /// A proposal is an `lcc.applications` row with
    /// `application_type = 'client_proposal'` — there is no `lcc.proposals`
    /// table and there never was one; see the module docs in `domain.rs` and
    /// the service README for the full evidence trail.
    ///
    /// Like `apply`, the row is a draft. The contract's `send-proposal` is a
    /// Tier-5 gated action that creates an approval; approval-svc owns that
    /// transition and this service does not fake it.
    #[allow(clippy::too_many_arguments)]
    pub async fn propose(
        &self,
        member_id: Uuid,
        opportunity_id: Uuid,
        title: &str,
        body: &str,
        price_cents: Option<i64>,
        currency: Option<String>,
        kb_ref_ids: Vec<Uuid>,
    ) -> Result<Application, Error> {
        if title.trim().is_empty() || body.trim().is_empty() {
            return Err(Error::Validation("title and body required".into()));
        }
        let opp = self.repo.get(member_id, opportunity_id).await?;
        let now = Utc::now();
        let payload = ProposalPayload {
            title: title.into(),
            body: body.into(),
            kb_refs: kb_ref_ids,
            price_cents,
            currency,
        };
        let a = Application {
            id: Uuid::new_v4(),
            member_id,
            opportunity_id,
            application_type: ApplicationType::ClientProposal,
            status: ApplicationStatus::Draft,
            position: Some(opp.title),
            submitted_at: None,
            response_received_at: None,
            payload: payload.to_json(),
            idempotency_key: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_application(&a).await?;
        Ok(a)
    }

    /// The proposals view: the member's `client_proposal` applications, read
    /// from `lcc.applications`. Same table, same tenant predicate.
    pub async fn list_proposals(&self, m: Uuid, limit: i64) -> Result<Vec<Application>, Error> {
        self.repo
            .list_applications(m, Some(ApplicationType::ClientProposal), limit)
            .await
    }
}
