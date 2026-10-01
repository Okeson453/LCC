//! Engagement repository.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ActionType, EngagementTask, InboxMessage, TaskStatus};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn list_tasks(
        &self,
        member_id: Uuid,
        status: Option<TaskStatus>,
        limit: i64,
    ) -> Result<Vec<EngagementTask>, Error> {
        let rows: Vec<(
            Uuid, Uuid, Option<Uuid>, Option<String>, String, String,
            Option<f64>, Option<DateTime<Utc>>, Option<String>, Vec<Uuid>,
            Option<DateTime<Utc>>, i32, DateTime<Utc>, DateTime<Utc>,
        )> = if let Some(s) = status {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, target_post_id,
                          action_type::TEXT, status::TEXT, priority_score, due_at,
                          draft, draft_pins, completed_at, version, created_at, updated_at
                   FROM lcc.engagement_replies
                   WHERE member_id = $1 AND status = $2::text
                   ORDER BY priority_score DESC NULLS LAST, due_at NULLS LAST
                   LIMIT $3"#,
            )
            .bind(member_id).bind(s.as_str()).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, target_post_id,
                          action_type::TEXT, status::TEXT, priority_score, due_at,
                          draft, draft_pins, completed_at, version, created_at, updated_at
                   FROM lcc.engagement_replies
                   WHERE member_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2"#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(|(id, member_id, contact_id, target_post_id, action_type, status,
                              priority_score, due_at, draft, draft_pins, completed_at,
                              version, created_at, updated_at)| {
            let action = serde_json::from_value::<ActionType>(serde_json::Value::String(action_type.clone()))
                .map_err(|e| Error::Internal(format!("action parse: {e}")))?;
            let s = serde_json::from_value::<TaskStatus>(serde_json::Value::String(status.clone()))
                .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
            Ok::<EngagementTask, Error>(EngagementTask {
                id, member_id, contact_id, target_post_id,
                action_type: action, status: s,
                priority_score, due_at, draft, draft_pins,
                completed_at, version, created_at, updated_at,
            })
        }).collect()
    }

    pub async fn insert_task(&self, t: &EngagementTask) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.engagement_replies
                  (id, member_id, contact_id, target_post_id, action_type,
                   status, priority_score, due_at, draft, draft_pins,
                   completed_at, version, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5::text,$6::text,$7,$8,$9,$10,$11,$12,$13,$14)"#,
        )
        .bind(t.id).bind(t.member_id).bind(t.contact_id).bind(&t.target_post_id)
        .bind(t.action_type.as_str()).bind(t.status.as_str())
        .bind(t.priority_score).bind(t.due_at).bind(&t.draft).bind(&t.draft_pins)
        .bind(t.completed_at).bind(t.version).bind(t.created_at).bind(t.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_task(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        draft: Option<&str>,
        draft_pins: Option<&[Uuid]>,
        status: Option<TaskStatus>,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET draft = COALESCE($4, draft),
                   draft_pins = COALESCE($5, draft_pins),
                   status = COALESCE($6::text, status),
                   version = version + 1,
                   updated_at = NOW()
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id).bind(id).bind(expected_version)
        .bind(draft).bind(draft_pins).bind(status.map(|s| s.as_str()))
        .fetch_one(&self.pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("task {id} v{expected_version}")),
            other => Error::Internal(format!("update task: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn complete_task(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET status = 'completed', completed_at = NOW(),
                   version = version + 1, updated_at = NOW()
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id).bind(id).bind(expected_version).fetch_one(&self.pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("task {id} v{expected_version}")),
            other => Error::Internal(format!("complete: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn inbox(
        &self,
        member_id: Uuid,
        limit: i64,
        unread_only: bool,
    ) -> Result<Vec<InboxMessage>, Error> {
        let rows: Vec<(
            Uuid, Uuid, String, Option<String>, String, bool, DateTime<Utc>,
        )> = if unread_only {
            sqlx::query_as(
                r#"SELECT id, contact_id, surface, subject, preview, unread, received_at
                   FROM lcc.engagement_inbox
                   WHERE member_id = $1 AND unread = TRUE
                   ORDER BY received_at DESC
                   LIMIT $2"#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as(
                r#"SELECT id, contact_id, surface, subject, preview, unread, received_at
                   FROM lcc.engagement_inbox
                   WHERE member_id = $1
                   ORDER BY received_at DESC
                   LIMIT $2"#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        };
        Ok(rows.into_iter().map(|(id, contact_id, surface, subject, preview, unread, received_at)| {
            InboxMessage { id, contact_id, surface, subject, preview, unread, received_at }
        }).collect())
    }

    pub async fn mark_inbox_read(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        sqlx::query("UPDATE lcc.engagement_inbox SET unread = FALSE
                     WHERE member_id = $1 AND id = $2")
            .bind(member_id).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn dismiss_task(&self, member_id: Uuid, id: Uuid, expected_version: i32) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET status = 'skipped', version = version + 1, updated_at = NOW()
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id).bind(id).bind(expected_version).fetch_one(&self.pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("task {id} v{expected_version}")),
            other => Error::Internal(format!("dismiss: {other}")),
        })?;
        Ok(row.0)
    }
}
