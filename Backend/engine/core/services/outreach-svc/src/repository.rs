//! Outreach repository.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Sequence, SequenceStatus, SequenceStep, StepKind, Template, TemplateStep,
};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pub pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn list_sequences(
        &self,
        member_id: Uuid,
        status: Option<SequenceStatus>,
        limit: i64,
    ) -> Result<Vec<Sequence>, Error> {
        let rows: Vec<(
            Uuid, Uuid, Uuid, Option<Uuid>, String, i32,
            Option<String>, Option<DateTime<Utc>>, i32, DateTime<Utc>, DateTime<Utc>,
        )> = if let Some(s) = status {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, template_id, status::TEXT,
                          current_step, paused_reason, last_step_sent_at,
                          version, created_at, updated_at
                   FROM lcc.sequences
                   WHERE member_id = $1 AND status = $2::text
                   ORDER BY created_at DESC
                   LIMIT $3"#,
            )
            .bind(member_id).bind(s.as_str()).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, template_id, status::TEXT,
                          current_step, paused_reason, last_step_sent_at,
                          version, created_at, updated_at
                   FROM lcc.sequences
                   WHERE member_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2"#,
            )
            .bind(member_id).bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter().map(|(id, member_id, contact_id, template_id, status,
                              current_step, paused_reason, last_step_sent_at,
                              version, created_at, updated_at)| {
            let st = serde_json::from_value::<SequenceStatus>(serde_json::Value::String(status.clone()))
                .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
            Ok::<Sequence, Error>(Sequence {
                id, member_id, contact_id, template_id, status: st,
                current_step, paused_reason, last_step_sent_at,
                version, created_at, updated_at,
            })
        }).collect()
    }

    pub async fn insert_sequence(&self, s: &Sequence) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.sequences
                  (id, member_id, contact_id, template_id, status, current_step,
                   paused_reason, last_step_sent_at, version, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5::text,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(s.id).bind(s.member_id).bind(s.contact_id).bind(s.template_id)
        .bind(s.status.as_str()).bind(s.current_step).bind(&s.paused_reason)
        .bind(s.last_step_sent_at).bind(s.version).bind(s.created_at).bind(s.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn update_status(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        new_status: SequenceStatus,
        paused_reason: Option<&str>,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"UPDATE lcc.sequences
               SET status = $4::text, paused_reason = $5,
                   version = version + 1, updated_at = NOW()
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version"#,
        )
        .bind(member_id).bind(id).bind(expected_version)
        .bind(new_status.as_str()).bind(paused_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Conflict(format!("sequence {id} v{expected_version}")),
            other => Error::Internal(format!("update seq: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn list_steps(
        &self,
        sequence_id: Uuid,
    ) -> Result<Vec<SequenceStep>, Error> {
        let rows: Vec<(
            Uuid, Uuid, i32, String, Option<String>, String, Option<String>,
            Uuid, Option<NaiveDateTime>, Option<DateTime<Utc>>,
            Option<DateTime<Utc>>, DateTime<Utc>,
        )> = sqlx::query_as(
            r#"SELECT id, sequence_id, step_index, kind::TEXT, subject,
                      body, rendered_body_hash, contact_id, scheduled_at,
                      sent_at, response_received_at, updated_at
               FROM lcc.sequence_steps
               WHERE sequence_id = $1
               ORDER BY step_index ASC"#,
        )
        .bind(sequence_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|(id, sequence_id, step_index, kind, subject, body,
                              rendered_body_hash, contact_id, scheduled_at,
                              sent_at, response_received_at, updated_at)| {
            let k = serde_json::from_value::<StepKind>(serde_json::Value::String(kind.clone()))
                .map_err(|e| Error::Internal(format!("kind parse: {e}")))?;
            Ok::<SequenceStep, Error>(SequenceStep {
                id, sequence_id, step_index, kind: k, subject, body,
                rendered_body_hash, contact_id, scheduled_at,
                sent_at, response_received_at, updated_at,
            })
        }).collect()
    }

    pub async fn mark_step_sent(&self, step_id: Uuid) -> Result<(), Error> {
        sqlx::query(
            "UPDATE lcc.sequence_steps SET sent_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(step_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn record_step_reply(&self, step_id: Uuid) -> Result<(), Error> {
        sqlx::query(
            "UPDATE lcc.sequence_steps SET response_received_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(step_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_template(&self, t: &Template) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.outreach_templates
                  (id, member_id, name, description, steps, is_published, version,
                   created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5::jsonb,$6,$7,$8,$9)"#,
        )
        .bind(t.id).bind(t.member_id).bind(&t.name).bind(&t.description)
        .bind(serde_json::to_value(&t.steps)?).bind(t.is_published).bind(t.version)
        .bind(t.created_at).bind(t.updated_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_templates(&self, member_id: Uuid) -> Result<Vec<Template>, Error> {
        let rows: Vec<(
            Uuid, Uuid, String, Option<String>, JsonValue, bool, i32,
            DateTime<Utc>, DateTime<Utc>,
        )> = sqlx::query_as(
            r#"SELECT id, member_id, name, description, steps, is_published, version,
                      created_at, updated_at
               FROM lcc.outreach_templates
               WHERE member_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(member_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|(id, member_id, name, description, steps, is_published,
                              version, created_at, updated_at)| {
            Ok::<Template, Error>(Template {
                id, member_id, name, description,
                steps: serde_json::from_value::<Vec<TemplateStep>>(steps)
                    .map_err(|e| Error::Internal(format!("steps parse: {e}")))?,
                is_published, version, created_at, updated_at,
            })
        }).collect()
    }
}
