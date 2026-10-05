//! Audit-svc service.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::AuditEventRow;
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
}

impl Service {
    pub fn new(repo: PgRepository) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        event_name: Option<&str>,
        producer_service: Option<&str>,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        limit: i64,
    ) -> Result<Vec<AuditEventRow>, Error> {
        self.repo
            .list(member_id, event_name, producer_service, start, end, limit)
            .await
    }

    pub async fn get(&self, id: Uuid) -> Result<AuditEventRow, Error> {
        self.repo.get(id).await
    }
}
