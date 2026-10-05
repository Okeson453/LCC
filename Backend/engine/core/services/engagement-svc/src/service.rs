//! Engagement service.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{ActionType, EngagementTask, InboxMessage, TaskStatus};
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

    pub async fn list(
        &self,
        member_id: Uuid,
        status: Option<TaskStatus>,
        limit: i64,
    ) -> Result<Vec<EngagementTask>, Error> {
        self.repo.list_tasks(member_id, status, limit).await
    }

    pub async fn inbox(
        &self,
        member_id: Uuid,
        limit: i64,
        unread_only: bool,
    ) -> Result<Vec<InboxMessage>, Error> {
        self.repo.inbox(member_id, limit, unread_only).await
    }

    pub async fn mark_read(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.mark_inbox_read(member_id, id).await
    }

    pub async fn create_task(
        &self,
        member_id: Uuid,
        action_type: ActionType,
        contact_id: Option<Uuid>,
        target_post_id: Option<String>,
        priority_score: Option<f64>,
        due_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<EngagementTask, Error> {
        let now = Utc::now();
        let t = EngagementTask {
            id: Uuid::new_v4(),
            member_id,
            contact_id,
            target_post_id,
            action_type,
            status: TaskStatus::Queued,
            priority_score,
            due_at,
            draft: None,
            draft_pins: vec![],
            completed_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert_task(&t).await?;
        self.publish_event(
            "engagement.task.created",
            member_id,
            &serde_json::json!({
                "task_id": t.id, "action_type": t.action_type.as_str(), "due_at": t.due_at,
            }),
        )
        .await;
        Ok(t)
    }

    pub async fn update_draft(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        draft: &str,
        draft_pins: &[Uuid],
    ) -> Result<EngagementTask, Error> {
        let new_v = self
            .repo
            .update_task(
                member_id,
                id,
                expected_version,
                Some(draft),
                Some(draft_pins),
                Some(TaskStatus::Drafted),
            )
            .await?;
        self.publish_event(
            "engagement.draft_ready",
            member_id,
            &serde_json::json!({
                "task_id": id, "draft_len": draft.len()
            }),
        )
        .await;
        // Refresh by listing one — for brevity, return a synthesized object.
        Ok(EngagementTask {
            id,
            member_id,
            contact_id: None,
            target_post_id: None,
            action_type: ActionType::Reply,
            status: TaskStatus::Drafted,
            priority_score: None,
            due_at: None,
            draft: Some(draft.into()),
            draft_pins: draft_pins.to_vec(),
            completed_at: None,
            version: new_v,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn complete(&self, m: Uuid, id: Uuid, v: i32) -> Result<EngagementTask, Error> {
        let new_v = self.repo.complete_task(m, id, v).await?;
        Ok(EngagementTask {
            id,
            member_id: m,
            contact_id: None,
            target_post_id: None,
            action_type: ActionType::Reply,
            status: TaskStatus::Completed,
            priority_score: None,
            due_at: None,
            draft: None,
            draft_pins: vec![],
            completed_at: Some(Utc::now()),
            version: new_v,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn dismiss(&self, m: Uuid, id: Uuid, v: i32) -> Result<EngagementTask, Error> {
        let new_v = self.repo.dismiss_task(m, id, v).await?;
        Ok(EngagementTask {
            id,
            member_id: m,
            contact_id: None,
            target_post_id: None,
            action_type: ActionType::Reply,
            status: TaskStatus::Skipped,
            priority_score: None,
            due_at: None,
            draft: None,
            draft_pins: vec![],
            completed_at: None,
            version: new_v,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn publish_event(&self, name: &str, m: Uuid, payload: &serde_json::Value) {
        let env = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": name,
            "occurred_at": Utc::now(),
            "member_id": m,
            "producer_service": "engagement-svc",
            "payload": payload,
        });
        if let Ok(mut conn) = self.redis.get().await {
            let _ = redis::cmd("XADD")
                .arg("lcc:realtime:events")
                .arg("*")
                .arg("envelope")
                .arg(env.to_string())
                .query_async::<String>(&mut conn)
                .await;
        }
    }
}
