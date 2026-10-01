//! Opportunity service.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    Application, Opportunity, OpportunityStatus, Proposal, ProposalStatus, Source,
};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    #[allow(dead_code)]
    redis: deadpool_redis::Pool,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }

    pub async fn list(
        &self, m: Uuid, s: Option<OpportunityStatus>, limit: i64,
    ) -> Result<Vec<Opportunity>, Error> {
        self.repo.list(m, s, limit).await
    }

    pub async fn discover(
        &self,
        member_id: Uuid,
        title: &str,
        source: Source,
        company_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<Opportunity, Error> {
        if title.trim().is_empty() {
            return Err(Error::Validation("title required".into()));
        }
        let now = Utc::now();
        let o = Opportunity {
            id: Uuid::new_v4(),
            member_id,
            company_id,
            title: title.into(),
            source,
            status: OpportunityStatus::Discovered,
            fit_score: None,
            discovered_at: now,
            last_evaluated_at: None,
            metadata,
            version: 1,
        };
        self.repo.insert(&o).await?;
        Ok(o)
    }

    pub async fn qualify(
        &self,
        m: Uuid,
        id: Uuid,
        v: i32,
        fit_score: f64,
        next: OpportunityStatus,
    ) -> Result<Opportunity, Error> {
        if !(0.0..=1.0).contains(&fit_score) {
            return Err(Error::Validation("fit_score must be 0..=1".into()));
        }
        let new_v = self.repo.qualify(m, id, v, fit_score, next).await?;
        let mut o = self.repo.get(m, id).await?;
        o.version = new_v;
        Ok(o)
    }

    pub async fn apply(
        &self,
        member_id: Uuid,
        opportunity_id: Uuid,
        cover_letter: Option<String>,
        resume_doc_id: Option<Uuid>,
    ) -> Result<Application, Error> {
        let now = Utc::now();
        let a = Application {
            id: Uuid::new_v4(),
            member_id,
            opportunity_id,
            status: OpportunityStatus::Applied,
            submitted_at: now,
            resume_doc_id,
            cover_letter,
            version: 1,
        };
        self.repo.insert_application(&a).await?;
        Ok(a)
    }

    pub async fn list_applications(&self, m: Uuid, limit: i64) -> Result<Vec<Application>, Error> {
        self.repo.list_applications(m, limit).await
    }

    pub async fn propose(
        &self,
        member_id: Uuid,
        opportunity_id: Uuid,
        title: &str,
        body: &str,
        price_cents: Option<i64>,
        currency: Option<String>,
        kb_ref_ids: Vec<Uuid>,
    ) -> Result<Proposal, Error> {
        if title.trim().is_empty() || body.trim().is_empty() {
            return Err(Error::Validation("title and body required".into()));
        }
        let now = Utc::now();
        let p = Proposal {
            id: Uuid::new_v4(),
            member_id,
            opportunity_id,
            title: title.into(),
            body: body.into(),
            price_cents,
            currency,
            status: ProposalStatus::Draft,
            kb_ref_ids,
            sent_at: None,
            version: 1,
            created_at: now,
        };
        self.repo.insert_proposal(&p).await?;
        Ok(p)
    }

    pub async fn list_proposals(&self, m: Uuid, limit: i64) -> Result<Vec<Proposal>, Error> {
        self.repo.list_proposals(m, limit).await
    }
}
