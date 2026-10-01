//! KB-svc service layer.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{EmbeddingStatus, KbKind, KbRecord};
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
        kind: Option<KbKind>,
        limit: i64,
    ) -> Result<Vec<KbRecord>, Error> {
        self.repo.list(member_id, kind, limit, None).await
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<KbRecord, Error> {
        self.repo.get(member_id, id).await
    }

    pub async fn create(
        &self,
        member_id: Uuid,
        kind: KbKind,
        title: &str,
        body: &str,
        tags: Vec<String>,
        source: Option<String>,
    ) -> Result<KbRecord, Error> {
        if title.trim().is_empty() {
            return Err(Error::Validation("title is required".into()));
        }
        if body.trim().is_empty() {
            return Err(Error::Validation("body is required".into()));
        }
        let now = Utc::now();
        let r = KbRecord {
            id: Uuid::new_v4(),
            member_id,
            kind,
            title: title.into(),
            body: body.into(),
            source,
            tags,
            embedding_id: None,
            embedding_status: EmbeddingStatus::Pending,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert(&r).await?;
        Ok(r)
    }

    pub async fn update(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        title: &str,
        body: &str,
        tags: &[String],
        source: Option<&str>,
    ) -> Result<KbRecord, Error> {
        let new_version = self
            .repo
            .update(member_id, id, expected_version, title, body, tags, source)
            .await?;
        self.repo.get(member_id, id).await.map(|mut r| {
            r.version = new_version;
            r
        })
    }

    pub async fn delete(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.repo.delete(member_id, id).await
    }

    /// Trigger re-embedding. The actual work happens in the kb-worker;
    /// we just flip status to `pending` and let the worker pick it up.
    pub async fn reembed(&self, member_id: Uuid, id: Uuid) -> Result<i32, Error> {
        let new_version = self.repo.mark_reembed(member_id, id).await?;
        // Publish a realtime event for the dashboard's KB panel.
        let mut conn = self.redis.get().await.map_err(|e| Error::Upstream(e.to_string()))?;
        let envelope = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": "kb.reembed.queued",
            "member_id": member_id,
            "payload": { "record_id": id, "version": new_version }
        });
        let _ = redis::cmd("XADD")
            .arg("lcc:realtime:events")
            .arg("*")
            .arg("envelope")
            .arg(envelope.to_string())
            .query_async::<String>(&mut conn)
            .await;
        Ok(new_version)
    }

    /// Worker callback: mark a record as embedded/failed.
    pub async fn set_embedding_status(
        &self,
        id: Uuid,
        status: EmbeddingStatus,
        embedding_id: Option<String>,
    ) -> Result<(), Error> {
        self.repo
            .set_embedding_status(id, status, embedding_id.as_deref())
            .await
    }
}
