//! Orchestrator service layer.
//!
//! Exposes the briefing assembly + event publication responsibilities.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::Briefing;
use crate::error::Error;
use crate::events_publisher::{publish_event, EventEnvelope};
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    redis: deadpool_redis::Pool,
    producer: String,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self {
            repo,
            redis,
            producer: "orchestrator".into(),
        }
    }

    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn assemble_briefing(
        &self,
        member_id: Uuid,
        date: NaiveDate,
    ) -> Result<Briefing, Error> {
        let briefing = self.repo.assemble_briefing(member_id, date).await?;

        // Emit the realtime briefing.refresh event so connected clients
        // can refetch the new payload. (Per contract, the consumer can
        // also receive `briefing.section.updated` for partial updates, but
        // a full refresh is the simplest and most-testable starting point.)
        let mut envelope = EventEnvelope::new(
            "briefing.refresh",
            member_id,
            &self.producer,
            serde_json::json!({
                "date": date.to_string(),
                "approvals_due": briefing.sections.approvals_due.len(),
                "hot_opportunities": briefing.sections.hot_opportunities.len(),
                "engagement": briefing.sections.engagement.len(),
                "followups": briefing.sections.followups.len(),
            }),
        );
        envelope.trace_id = uuid::Uuid::new_v4();

        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| Error::Upstream(format!("redis pool: {e}")))?;
        let _ = publish_event(&mut conn, &envelope).await; // best-effort

        Ok(briefing)
    }
}
