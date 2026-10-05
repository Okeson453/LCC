//! Approval service.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    Approval, ApprovalStatus, BulkDecideInput, BulkDecideResult, RequestApprovalInput,
};
use crate::error::Error;
use crate::repository::PgRepository;

pub struct Service {
    repo: PgRepository,
    redis: deadpool_redis::Pool,
}

impl Service {
    pub fn new(repo: PgRepository, redis: deadpool_redis::Pool) -> Self {
        Self { repo, redis }
    }
    pub fn repo(&self) -> &PgRepository {
        &self.repo
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        status: Option<ApprovalStatus>,
        limit: i64,
    ) -> Result<Vec<Approval>, Error> {
        self.repo.list(member_id, status, limit).await
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<Approval, Error> {
        self.repo.get(member_id, id).await
    }

    pub async fn request(
        &self,
        member_id: Uuid,
        requester_id: Uuid,
        input: RequestApprovalInput,
    ) -> Result<Approval, Error> {
        if input.tier < 1 || input.tier > 3 {
            return Err(Error::Validation("tier must be 1..=3".into()));
        }
        if input.resource_type.trim().is_empty() {
            return Err(Error::Validation("resource_type required".into()));
        }
        let now = Utc::now();
        let a = Approval {
            id: Uuid::new_v4(),
            member_id,
            resource_type: input.resource_type,
            resource_id: input.resource_id,
            requested_action: input.requested_action,
            tier: input.tier,
            rule_version: input.rule_version,
            status: ApprovalStatus::Pending,
            requested_by: requester_id,
            decided_reason: None,
            reviewer_ids: vec![],
            expires_at: input.ttl_hours.map(|h| now + chrono::Duration::hours(h)),
            version: 1,
            created_at: now,
            decided_at: None,
        };
        self.repo.insert(&a).await?;
        // Publish approval.created event.
        let envelope = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": "approval.created",
            "occurred_at": now,
            "member_id": member_id,
            "producer_service": "approval-svc",
            "payload": {
                "approval_id": a.id,
                "resource_type": a.resource_type,
                "resource_id": a.resource_id,
                "tier": a.tier,
            }
        });
        if let Ok(mut conn) = self.redis.get().await {
            let _ = redis::cmd("XADD")
                .arg("lcc:realtime:events")
                .arg("*")
                .arg("envelope")
                .arg(envelope.to_string())
                .query_async::<String>(&mut conn)
                .await;
        }
        Ok(a)
    }

    pub async fn decide(
        &self,
        member_id: Uuid,
        reviewer_id: Uuid,
        id: Uuid,
        new_status: ApprovalStatus,
        reason: Option<&str>,
        expected_version: i32,
    ) -> Result<Approval, Error> {
        let new_v = self
            .repo
            .decide(
                member_id,
                id,
                expected_version,
                new_status,
                reason,
                reviewer_id,
            )
            .await?;
        let mut a = self.repo.get(member_id, id).await?;
        a.version = new_v;
        Ok(a)
    }

    pub async fn bulk_decide(
        &self,
        member_id: Uuid,
        reviewer_id: Uuid,
        input: BulkDecideInput,
    ) -> Result<BulkDecideResult, Error> {
        let outcomes = self
            .repo
            .bulk_decide(
                member_id,
                &input.ids,
                input.decision,
                &input.reason,
                reviewer_id,
                input.expected_version,
            )
            .await?;
        let approved = outcomes
            .iter()
            .filter(|(_, ok)| *ok && input.decision == ApprovalStatus::Approved)
            .count();
        let rejected = outcomes
            .iter()
            .filter(|(_, ok)| *ok && input.decision == ApprovalStatus::Rejected)
            .count();
        let conflicts = outcomes
            .into_iter()
            .filter_map(|(id, ok)| if !ok { Some(id) } else { None })
            .collect();
        Ok(BulkDecideResult {
            approved,
            rejected,
            conflicts,
        })
    }
}
