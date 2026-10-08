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

    /// Idempotent by construction: marking an already-read message changes no
    /// rows, and an id that is not this member's changes no rows either. Both
    /// answer 204, so the endpoint is not an existence oracle.
    pub async fn mark_read(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.mark_inbox_read(member_id, id).await?;
        Ok(())
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
            // `queued` is the design §11.12 / contract / proto starting state.
            status: TaskStatus::Queued,
            priority_score,
            due_at,
            draft_body: None,
            completed_at: None,
            version: 1,
            created_at: now,
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

    /// Store a draft and move the task to `drafted`.
    ///
    /// The response is the row as stored. The previous version built the
    /// response in memory — `contact_id: None`, `action_type: Reply`,
    /// `created_at: Utc::now()`, `updated_at: Utc::now()` — so a caller was told
    /// the task was a reply with no contact and was created just now, whatever
    /// the database actually held. Re-reading is the only version of this that
    /// cannot lie.
    pub async fn update_draft(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        draft_body: &str,
    ) -> Result<EngagementTask, Error> {
        let new_version = self
            .repo
            .update_task(
                member_id,
                id,
                expected_version,
                Some(draft_body),
                Some(TaskStatus::Drafted),
            )
            .await?;
        let version = match new_version {
            Some(v) => v,
            None => {
                return Err(self
                    .explain_no_update(member_id, id, expected_version)
                    .await)
            }
        };
        self.publish_event(
            "engagement.draft_ready",
            member_id,
            &serde_json::json!({ "task_id": id, "draft_len": draft_body.len() }),
        )
        .await;
        debug_assert_eq!(version, expected_version + 1);
        self.require_task(member_id, id).await
    }

    /// Mark the reply as sent.
    ///
    /// The canonical terminal state is `sent` (design §11.12, the contract
    /// enum, the proto enum); the previous `'completed'` is in none of them.
    pub async fn complete(&self, m: Uuid, id: Uuid, v: i32) -> Result<EngagementTask, Error> {
        let new_version = self.repo.complete_task(m, id, v).await?;
        if new_version.is_none() {
            return Err(self.explain_no_update(m, id, v).await);
        }
        self.require_task(m, id).await
    }

    /// Mark the task dismissed — the contract's `dismissed`, not `skipped`.
    pub async fn dismiss(&self, m: Uuid, id: Uuid, v: i32) -> Result<EngagementTask, Error> {
        let new_version = self.repo.dismiss_task(m, id, v).await?;
        if new_version.is_none() {
            return Err(self.explain_no_update(m, id, v).await);
        }
        self.require_task(m, id).await
    }

    /// Re-read a task the caller owns. 404 if it is not theirs or is gone.
    async fn require_task(&self, member_id: Uuid, id: Uuid) -> Result<EngagementTask, Error> {
        self.repo
            .get_task(member_id, id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("engagement task {id}")))
    }

    /// Turn "the guarded UPDATE matched no row" into the right status.
    ///
    /// The UPDATE returns nothing for three different reasons, and answering
    /// the same way for all of them (as the previous code did, with a blanket
    /// 409) turns the endpoint into an existence oracle: a caller could send a
    /// bogus version against a guessed task id and learn from 409-vs-404 that
    /// the task exists. So the row is re-checked with the same `member_id`
    /// predicate the UPDATE used: absent or not owned is 404, owned with a
    /// different `version` is 409.
    async fn explain_no_update(&self, member_id: Uuid, id: Uuid, expected_version: i32) -> Error {
        match self.repo.get_task(member_id, id).await {
            Ok(Some(task)) => Error::Conflict(format!(
                "engagement task {id} is at version {}, not {expected_version}",
                task.version
            )),
            Ok(None) => Error::NotFound(format!("engagement task {id}")),
            // A read failure here must not be reported as a version conflict:
            // that would tell a caller their task exists when we do not know.
            Err(e) => e,
        }
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
