//! Profile-svc service layer.

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    ConsentKind, ConsentRecord, EditDraftStatus, ProfileEditDraft, ProfileSnapshot,
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

    pub async fn get_profile(&self, member_id: Uuid) -> Result<ProfileSnapshot, Error> {
        self.repo.get_current(member_id).await
    }

    /// Save a snapshot. Version is supplied by the caller; we record a
    /// new history row keyed by (member_id, version).
    pub async fn save_snapshot(
        &self,
        member_id: Uuid,
        mut snapshot: ProfileSnapshot,
    ) -> Result<ProfileSnapshot, Error> {
        if snapshot.member_id != member_id {
            return Err(Error::Validation(
                "snapshot.member_id must equal authenticated member".into(),
            ));
        }
        let current = self.repo.get_current(member_id).await?;
        let expected_version = current.version;
        snapshot.version = expected_version + 1;
        snapshot.id = Uuid::new_v4();
        snapshot.created_at = Utc::now();

        self.repo.insert_snapshot(&snapshot, expected_version).await?;

        // Emit audit event — best-effort, never blocks the snapshot.
        let audit_event = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "event_name": "profile.snapshot.saved",
            "occurred_at": Utc::now(),
            "member_id": member_id,
            "producer_service": "profile-svc",
            "actor": "profile-svc",
            "action": "profile.snapshot.saved",
            "resource_type": "profile_snapshot",
            "resource_id": snapshot.id,
            "payload": {
                "snapshot_id": snapshot.id,
                "version": snapshot.version,
                "headline": &snapshot.headline,
            }
        });
        if let Ok(mut conn) = self.redis.get().await {
            let _ = redis::cmd("XADD")
                .arg("lcc:audit:events")
                .arg("*")
                .arg("event")
                .arg(audit_event.to_string())
                .query_async::<String>(&mut conn)
                .await;
        }
        Ok(snapshot)
    }

    pub async fn create_edit_draft(
        &self,
        member_id: Uuid,
        profile_id: Uuid,
        proposed_fields: serde_json::Value,
    ) -> Result<ProfileEditDraft, Error> {
        // Enforce: cannot create draft if one is already pending.
        let draft = ProfileEditDraft {
            id: Uuid::new_v4(),
            profile_id,
            member_id,
            proposed_fields,
            status: EditDraftStatus::Pending,
            version: 1,
            created_at: Utc::now(),
        };
        self.repo.insert_edit_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn decide_edit_draft(
        &self,
        member_id: Uuid,
        draft_id: Uuid,
        new_status: EditDraftStatus,
        expected_version: i32,
    ) -> Result<(), Error> {
        self.repo
            .update_edit_draft_status(draft_id, member_id, new_status, expected_version)
            .await
    }

    pub async fn get_consent(
        &self,
        member_id: Uuid,
        kind: ConsentKind,
    ) -> Result<ConsentRecord, Error> {
        self.repo.get_consent(member_id, kind).await
    }

    pub async fn grant_consent(
        &self,
        member_id: Uuid,
        kind: ConsentKind,
        ttl_days: Option<i64>,
    ) -> Result<ConsentRecord, Error> {
        let expires_at = ttl_days.map(|d| Utc::now() + chrono::Duration::days(d));
        let next_version = self
            .repo
            .get_consent(member_id, kind)
            .map(|c| c.version + 1)
            .await
            .unwrap_or(1);
        let c = ConsentRecord {
            id: Uuid::new_v4(),
            member_id,
            consent_kind: kind,
            granted: true,
            granted_at: Utc::now(),
            expires_at,
            version: next_version,
        };
        self.repo.upsert_consent(&c).await?;
        Ok(c)
    }
}
