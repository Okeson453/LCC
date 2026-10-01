//! KB-svc repository — sqlx queries against lcc.kb_records.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{EmbeddingStatus, KbKind, KbRecord};
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        kind: Option<KbKind>,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<KbRecord>, Error> {
        let rows: Vec<(
            Uuid,
            Uuid,
            String,
            String,
            String,
            Option<String>,
            Vec<String>,
            Option<String>,
            String,
            i32,
            DateTime<Utc>,
            DateTime<Utc>,
        )> = if let Some(kind) = kind {
            sqlx::query_as(
                r#"
                SELECT id, member_id, kind::TEXT, title, body, source, tags,
                       embedding_id, embedding_status::TEXT, version,
                       created_at, updated_at
                FROM lcc.kb_records
                WHERE member_id = $1 AND kind = $2::text
                ORDER BY created_at DESC
                LIMIT $3
                "#,
            )
            .bind(member_id)
            .bind(kind.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, member_id, kind::TEXT, title, body, source, tags,
                       embedding_id, embedding_status::TEXT, version,
                       created_at, updated_at
                FROM lcc.kb_records
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

        rows.into_iter()
            .map(map_row)
            .collect::<Result<Vec<_>, _>>()
            .map(|v: Vec<KbRecord>| {
                let _ = cursor;
                v
            })
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<KbRecord, Error> {
        let row: (
            Uuid,
            Uuid,
            String,
            String,
            String,
            Option<String>,
            Vec<String>,
            Option<String>,
            String,
            i32,
            DateTime<Utc>,
            DateTime<Utc>,
        ) = sqlx::query_as(
            r#"
            SELECT id, member_id, kind::TEXT, title, body, source, tags,
                   embedding_id, embedding_status::TEXT, version,
                   created_at, updated_at
            FROM lcc.kb_records
            WHERE member_id = $1 AND id = $2
            "#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("kb record {id}")),
            other => Error::Internal(format!("get kb: {other}")),
        })?;
        Ok(map_row(row)?)
    }

    pub async fn insert(&self, r: &KbRecord) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO lcc.kb_records
                (id, member_id, kind, title, body, source, tags,
                 embedding_id, embedding_status, version, created_at, updated_at)
            VALUES ($1, $2, $3::text, $4, $5, $6, $7, $8, $9::text, $10, $11, $12)
            "#,
        )
        .bind(r.id)
        .bind(r.member_id)
        .bind(r.kind.as_str())
        .bind(&r.title)
        .bind(&r.body)
        .bind(&r.source)
        .bind(&r.tags)
        .bind(&r.embedding_id)
        .bind(r.embedding_status.as_str())
        .bind(r.version)
        .bind(r.created_at)
        .bind(r.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
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
    ) -> Result<i32, Error> {
        let updated = sqlx::query(
            r#"
            UPDATE lcc.kb_records
            SET title = $4, body = $5, tags = $6, source = $7,
                version = version + 1, updated_at = NOW(),
                embedding_status = 'stale'
            WHERE member_id = $1 AND id = $2 AND version = $3
            "#,
        )
        .bind(member_id)
        .bind(id)
        .bind(expected_version)
        .bind(title)
        .bind(body)
        .bind(tags)
        .bind(source)
        .execute(&self.pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(Error::Conflict(format!(
                "kb record {id} v{expected_version} mismatch"
            )));
        }
        Ok(expected_version + 1)
    }

    pub async fn delete(&self, member_id: Uuid, id: Uuid) -> Result<(), Error> {
        let deleted = sqlx::query("DELETE FROM lcc.kb_records WHERE member_id = $1 AND id = $2")
            .bind(member_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if deleted.rows_affected() == 0 {
            return Err(Error::NotFound(format!("kb record {id}")));
        }
        Ok(())
    }

    /// Re-embed: set the embedding status to `pending` and bump the version
    /// so callers re-running this loop get a fresh version number to compare.
    /// The actual embedding job is performed by the worker (not in this service).
    pub async fn mark_reembed(&self, member_id: Uuid, id: Uuid) -> Result<i32, Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE lcc.kb_records
            SET embedding_status = 'pending', version = version + 1, updated_at = NOW()
            WHERE member_id = $1 AND id = $2
            RETURNING version
            "#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("kb record {id}")),
            other => Error::Internal(format!("reembed: {other}")),
        })?;
        Ok(row.0)
    }

    pub async fn set_embedding_status(
        &self,
        id: Uuid,
        status: EmbeddingStatus,
        embedding_id: Option<&str>,
    ) -> Result<(), Error> {
        let res = sqlx::query(
            r#"
            UPDATE lcc.kb_records
            SET embedding_status = $2::text, embedding_id = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(embedding_id)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(Error::NotFound(format!("kb record {id}")));
        }
        Ok(())
    }
}

fn map_row(
    row: (
        Uuid,
        Uuid,
        String,
        String,
        String,
        Option<String>,
        Vec<String>,
        Option<String>,
        String,
        i32,
        DateTime<Utc>,
        DateTime<Utc>,
    ),
) -> Result<KbRecord, Error> {
    use serde_json::Value;
    let kind = serde_json::from_value::<KbKind>(Value::String(row.2.clone()))
        .map_err(|e| Error::Internal(format!("kind parse: {e}")))?;
    let embedding_status =
        serde_json::from_value::<EmbeddingStatus>(Value::String(row.8.clone()))
            .map_err(|e| Error::Internal(format!("status parse: {e}")))?;
    Ok(KbRecord {
        id: row.0,
        member_id: row.1,
        kind,
        title: row.3,
        body: row.4,
        source: row.5,
        tags: row.6,
        embedding_id: row.7,
        embedding_status,
        version: row.9,
        created_at: row.10,
        updated_at: row.11,
    })
}
