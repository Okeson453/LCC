//! network-crm-svc service.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    Company, Contact, Interaction, InteractionDirection, InteractionKind, StalenessReport,
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
    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn list_contacts(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Contact>, Error> {
        self.repo.list_contacts(member_id, q, limit).await
    }

    pub async fn get_contact(&self, m: Uuid, id: Uuid) -> Result<Contact, Error> {
        self.repo.get_contact(m, id).await
    }

    pub async fn create_contact(
        &self,
        member_id: Uuid,
        full_name: &str,
        title: Option<&str>,
        headline: Option<&str>,
        linkedin_url: Option<&str>,
        email: Option<&str>,
        company_id: Option<Uuid>,
        tags: Vec<String>,
        notes: Option<&str>,
    ) -> Result<Contact, Error> {
        if full_name.trim().is_empty() {
            return Err(Error::Validation("full_name required".into()));
        }
        let now = Utc::now();
        let c = Contact {
            id: Uuid::new_v4(),
            member_id,
            company_id,
            full_name: full_name.into(),
            title: title.map(str::to_string),
            headline: headline.map(str::to_string),
            linkedin_url: linkedin_url.map(str::to_string),
            email: email.map(str::to_string),
            connection_strength: 0,
            last_touched_at: None,
            last_interaction_kind: None,
            tags,
            notes: notes.map(str::to_string),
            stale_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_contact(&c).await?;
        Ok(c)
    }

    pub async fn update_contact(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        patch: ContactPatch,
    ) -> Result<Contact, Error> {
        let current = self.repo.get_contact(member_id, id).await?;
        let updated = Contact {
            id: current.id,
            member_id: current.member_id,
            company_id: patch.company_id.or(current.company_id),
            full_name: patch.full_name.unwrap_or(current.full_name),
            title: patch.title.or(current.title),
            headline: patch.headline.or(current.headline),
            linkedin_url: patch.linkedin_url.or(current.linkedin_url),
            email: patch.email.or(current.email),
            connection_strength: current.connection_strength,
            last_touched_at: current.last_touched_at,
            last_interaction_kind: current.last_interaction_kind,
            tags: patch.tags.unwrap_or(current.tags),
            notes: patch.notes.or(current.notes),
            stale_at: current.stale_at,
            version: current.version,
            created_at: current.created_at,
            updated_at: current.updated_at,
        };
        let new_v = self
            .repo
            .update_contact(member_id, id, expected_version, &updated)
            .await?;
        self.repo.get_contact(member_id, id).await.map(|mut c| {
            c.version = new_v;
            c
        })
    }

    pub async fn delete_contact(&self, m: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.delete_contact(m, id).await
    }

    pub async fn record_interaction(
        &self,
        member_id: Uuid,
        contact_id: Uuid,
        kind: InteractionKind,
        summary: &str,
        direction: InteractionDirection,
        channel: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<Interaction, Error> {
        let i = Interaction {
            id: Uuid::new_v4(),
            member_id,
            contact_id,
            kind,
            summary: summary.into(),
            occurred_at: Utc::now(),
            channel,
            direction,
            metadata,
        };
        self.repo.record_interaction(&i).await?;
        self.repo
            .touch_contact(member_id, contact_id, kind.as_str())
            .await?;
        Ok(i)
    }

    pub async fn list_companies(
        &self,
        member_id: Uuid,
        q: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Company>, Error> {
        self.repo.list_companies(member_id, q, limit).await
    }

    pub async fn create_company(
        &self,
        member_id: Uuid,
        name: &str,
        domain: Option<&str>,
        industry: Option<&str>,
        size_band: Option<&str>,
        funding_stage: Option<&str>,
        tech_stack: Vec<String>,
        trigger_events: Vec<String>,
        public_signals: Vec<String>,
        ttl_at: Option<chrono::NaiveDate>,
    ) -> Result<Company, Error> {
        if name.trim().is_empty() {
            return Err(Error::Validation("name required".into()));
        }
        let now = Utc::now();
        let c = Company {
            id: Uuid::new_v4(),
            member_id,
            name: name.into(),
            domain: domain.map(str::to_string),
            industry: industry.map(str::to_string),
            size_band: size_band.map(str::to_string),
            funding_stage: funding_stage.map(str::to_string),
            tech_stack,
            trigger_events,
            public_signals,
            ttl_at,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_company(&c).await?;
        Ok(c)
    }

    pub async fn staleness(
        &self,
        member_id: Uuid,
        days: i64,
    ) -> Result<StalenessReport, Error> {
        self.repo.staleness(member_id, days).await
    }

    pub async fn link(&self, m: Uuid, contact_id: Uuid, company_id: Uuid) -> Result<(), Error> {
        self.repo.link_contact_company(m, contact_id, company_id).await
    }
}

#[derive(Debug, Default)]
pub struct ContactPatch {
    pub full_name: Option<String>,
    pub title: Option<Option<String>>,
    pub headline: Option<Option<String>>,
    pub linkedin_url: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub company_id: Option<Option<Uuid>>,
}
