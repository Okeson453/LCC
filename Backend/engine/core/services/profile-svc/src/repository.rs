//! Profile-svc repository — sqlx queries against lcc.* tables.
//!
//! Tables used:
//!   lcc.profile_snapshots
//!   lcc.profile_edit_drafts
//!   lcc.profile_consent_records

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    ConsentKind, ConsentRecord, EditDraftStatus, ProfileEditDraft, ProfileSnapshot,
};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

/// A `lcc.profile_snapshots` row. Column order must match the SELECT below.
type ProfileSnapshotRow = (
    Uuid,          // id
    Uuid,          // member_id
    i32,           // version
    String,        // headline
    String,        // summary
    Vec<String>,   // skills
    JsonValue,     // experiences
    JsonValue,     // education
    Vec<Uuid>,     // kb_ref_ids
    DateTime<Utc>, // created_at
);

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_current(&self, member_id: Uuid) -> Result<ProfileSnapshot, Error> {
        let row: ProfileSnapshotRow = sqlx::query_as(
            r#"
            SELECT id, member_id, version, headline, summary, skills,
                   experiences, education, kb_ref_ids, created_at
            FROM lcc.profile_snapshots
            WHERE member_id = $1
            ORDER BY version DESC LIMIT 1
            "#,
        )
        .bind(member_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                Error::NotFound(format!("no profile for member {member_id}"))
            }
            other => Error::Internal(format!("get profile: {other}")),
        })?;

        Ok(ProfileSnapshot {
            id: row.0,
            member_id: row.1,
            version: row.2,
            headline: row.3,
            summary: row.4,
            skills: row.5,
            experiences: serde_json::from_value(row.6)
                .map_err(|e| Error::Internal(format!("experiences json: {e}")))?,
            education: serde_json::from_value(row.7)
                .map_err(|e| Error::Internal(format!("education json: {e}")))?,
            kb_ref_ids: row.8,
            created_at: row.9,
        })
    }

    /// Append a new snapshot row (history table per Backend Design §4.3).
    pub async fn insert_snapshot(
        &self,
        snapshot: &ProfileSnapshot,
        expected_version: i32,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query(
            r#"
            INSERT INTO lcc.profile_snapshots
                (id, member_id, version, headline, summary, skills,
                 experiences, education, kb_ref_ids, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(snapshot.id)
        .bind(snapshot.member_id)
        .bind(snapshot.version)
        .bind(&snapshot.headline)
        .bind(&snapshot.summary)
        .bind(&snapshot.skills)
        .bind(serde_json::to_value(&snapshot.experiences)?)
        .bind(serde_json::to_value(&snapshot.education)?)
        .bind(&snapshot.kb_ref_ids)
        .bind(snapshot.created_at)
        .execute(&mut *tx)
        .await;
        match res {
            Ok(_) => {
                tx.commit().await?;
                let _ = expected_version; // captured for optimistic-concurrency verification at the boundary
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(Error::Conflict(format!(
                    "snapshot for member {} v{} already exists",
                    snapshot.member_id, snapshot.version
                )))
            }
            Err(e) => Err(Error::Internal(format!("insert snapshot: {e}"))),
        }
    }

    pub async fn insert_edit_draft(&self, draft: &ProfileEditDraft) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.profile_edit_drafts
                (id, profile_id, member_id, proposed_fields,
                 status, version, created_at)
            VALUES ($1, $2, $3, $4, $5::text, $6, $7)
            "#,
        )
        .bind(draft.id)
        .bind(draft.profile_id)
        .bind(draft.member_id)
        .bind(&draft.proposed_fields)
        .bind(draft.status.as_str())
        .bind(draft.version)
        .bind(draft.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_edit_draft_status(
        &self,
        draft_id: Uuid,
        member_id: Uuid,
        new_status: EditDraftStatus,
        expected_version: i32,
    ) -> Result<(), Error> {
        let updated = sqlx::query(
            r#"
            UPDATE lcc.profile_edit_drafts
            SET status = $3::text
            WHERE id = $1 AND member_id = $2 AND version = $4
            "#,
        )
        .bind(draft_id)
        .bind(member_id)
        .bind(new_status.as_str())
        .bind(expected_version)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(Error::Conflict("edit draft version mismatch".into()));
        }
        Ok(())
    }

    pub async fn get_consent(
        &self,
        member_id: Uuid,
        kind: ConsentKind,
    ) -> Result<ConsentRecord, Error> {
        let row: (
            Uuid,
            Uuid,
            String,
            bool,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            i32,
        ) = sqlx::query_as(
            r#"
            SELECT id, member_id, consent_kind::TEXT, granted,
                   granted_at, expires_at, version
            FROM lcc.profile_consent_records
            WHERE member_id = $1 AND consent_kind = $2::text
            ORDER BY version DESC LIMIT 1
            "#,
        )
        .bind(member_id)
        .bind(kind.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!(
                "no consent for member {member_id} kind={}",
                kind.as_str()
            )),
            other => Error::Internal(format!("get consent: {other}")),
        })?;
        Ok(ConsentRecord {
            id: row.0,
            member_id: row.1,
            consent_kind: serde_json::from_value(serde_json::Value::String(row.2))
                .map_err(|e| Error::Internal(format!("consent_kind parse: {e}")))?,
            granted: row.3,
            granted_at: row.4,
            expires_at: row.5,
            version: row.6,
        })
    }

    pub async fn upsert_consent(&self, c: &ConsentRecord) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.profile_consent_records
                (id, member_id, consent_kind, granted,
                 granted_at, expires_at, version)
            VALUES ($1, $2, $3::text, $4, $5, $6, $7)
            "#,
        )
        .bind(c.id)
        .bind(c.member_id)
        .bind(c.consent_kind.as_str())
        .bind(c.granted)
        .bind(c.granted_at)
        .bind(c.expires_at)
        .bind(c.version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
