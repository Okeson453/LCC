//! Content-svc repository — sqlx queries against lcc.content_items.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ContentItem, ContentKind, ContentMetrics, ContentState, QualityCheckResult};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pub pool: PgPool,
}

/// One `lcc.content_items` row.
///
/// A named struct rather than a tuple: the projection has 19 columns and sqlx
/// only implements `FromRow` for tuples up to arity 16.
#[derive(Debug, sqlx::FromRow)]
pub struct ContentRow {
    pub id: Uuid,
    pub member_id: Uuid,
    pub state: String,
    pub title: String,
    pub body: String,
    pub rendered_body_hash: Option<String>,
    pub kind: String,
    pub topic: String,
    pub voice_style_kb_id: Option<Uuid>,
    pub pinned_kb_ids: Vec<Uuid>,
    pub metrics: JsonValue,
    pub quality_loop_count: i32,
    pub idempotency_key: Option<String>,
    pub expected_version: i32,
    pub version: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        state: Option<ContentState>,
        limit: i64,
    ) -> Result<Vec<ContentItem>, Error> {
        let rows: Vec<ContentRow> = if let Some(state) = state {
            sqlx::query_as(
                r#"
                SELECT id, member_id, state::TEXT, title, body, rendered_body_hash,
                       kind::TEXT, topic, voice_style_kb_id, pinned_kb_ids,
                       metrics, quality_loop_count, idempotency_key,
                       expected_version, version, scheduled_at, published_at,
                       created_at, updated_at
                FROM lcc.content_items
                WHERE member_id = $1 AND state = $2::text
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(member_id)
            .bind(state.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, member_id, state::TEXT, title, body, rendered_body_hash,
                       kind::TEXT, topic, voice_style_kb_id, pinned_kb_ids,
                       metrics, quality_loop_count, idempotency_key,
                       expected_version, version, scheduled_at, published_at,
                       created_at, updated_at
                FROM lcc.content_items
                WHERE member_id = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        rows.into_iter().map(map_row).collect()
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<ContentItem, Error> {
        let row = sqlx::query_as::<_, ContentRow>(
            r#"
            SELECT id, member_id, state::TEXT, title, body, rendered_body_hash,
                   kind::TEXT, topic, voice_style_kb_id, pinned_kb_ids,
                   metrics, quality_loop_count, idempotency_key,
                   expected_version, version, scheduled_at, published_at,
                   created_at, updated_at
            FROM lcc.content_items
            WHERE member_id = $1 AND id = $2
            "#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("content {id}")),
            other => Error::Internal(format!("get content: {other}")),
        })?;
        map_row(row)
    }

    pub async fn insert(&self, c: &ContentItem) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.content_items
                (id, member_id, state, title, body, rendered_body_hash,
                 kind, topic, voice_style_kb_id, pinned_kb_ids, metrics,
                 quality_loop_count, idempotency_key, expected_version, version,
                 scheduled_at, published_at, created_at, updated_at)
            VALUES ($1, $2, $3::text, $4, $5, $6, $7::text, $8,
                    $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(c.id)
        .bind(c.member_id)
        .bind(c.state.as_str())
        .bind(&c.title)
        .bind(&c.body)
        .bind(&c.rendered_body_hash)
        .bind(c.kind.as_str())
        .bind(&c.topic)
        .bind(c.voice_style_kb_id)
        .bind(&c.pinned_kb_ids)
        .bind(serde_json::to_value(&c.metrics)?)
        .bind(c.quality_loop_count)
        .bind(&c.idempotency_key)
        .bind(c.expected_version)
        .bind(c.version)
        .bind(c.scheduled_at)
        .bind(c.published_at)
        .bind(c.created_at)
        .bind(c.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_body(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        body: &str,
        rendered_hash: Option<&str>,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE lcc.content_items
            SET body = $4, rendered_body_hash = $5,
                version = version + 1,
                updated_at = NOW()
            WHERE member_id = $1 AND id = $2 AND version = $3
            RETURNING version
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(body)
        .bind(rendered_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                Error::Conflict(format!("content {id} v{expected_version} not found"))
            }
            other => Error::Internal(format!("update body: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn transition(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        new_state: ContentState,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE lcc.content_items
            SET state = $4::text,
                version = version + 1,
                scheduled_at = COALESCE($5, scheduled_at),
                published_at = CASE WHEN $4::text = 'published' THEN NOW() ELSE published_at END,
                updated_at = NOW()
            WHERE member_id = $1 AND id = $2 AND version = $3
            RETURNING version
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(new_state.as_str())
        .bind(scheduled_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                Error::Conflict(format!("content {id} v{expected_version} not found"))
            }
            other => Error::Internal(format!("transition: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn increment_quality_loop(&self, id: Uuid) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE lcc.content_items
            SET quality_loop_count = quality_loop_count + 1, updated_at = NOW()
            WHERE id = $1
            RETURNING quality_loop_count
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    pub async fn record_quality_check(
        &self,
        id: Uuid,
        q: &QualityCheckResult,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.content_quality_checks
                (id, content_id, passed, loop, issues, auto_fixes, evaluated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(q.passed)
        .bind(q.r#loop)
        .bind(&q.issues)
        .bind(&q.auto_fixes)
        .bind(q.evaluated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM lcc.content_items WHERE member_id = $1 AND id = $2")
            .bind(member_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

pub fn map_row(row: ContentRow) -> Result<ContentItem, Error> {
    use serde_json::Value;
    let state = serde_json::from_value::<ContentState>(Value::String(row.state.clone()))
        .map_err(|e| Error::Internal(format!("state parse: {e}")))?;
    let kind = serde_json::from_value::<ContentKind>(Value::String(row.kind.clone()))
        .map_err(|e| Error::Internal(format!("kind parse: {e}")))?;
    let metrics: ContentMetrics = serde_json::from_value(row.metrics.clone())
        .map_err(|e| Error::Internal(format!("metrics parse: {e}")))?;
    Ok(ContentItem {
        id: row.id,
        member_id: row.member_id,
        state,
        title: row.title,
        body: row.body,
        rendered_body_hash: row.rendered_body_hash,
        kind,
        topic: row.topic,
        voice_style_kb_id: row.voice_style_kb_id,
        pinned_kb_ids: row.pinned_kb_ids,
        metrics,
        quality_loop_count: row.quality_loop_count,
        idempotency_key: row.idempotency_key,
        expected_version: row.expected_version,
        version: row.version,
        scheduled_at: row.scheduled_at,
        published_at: row.published_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
