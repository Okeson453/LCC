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

/// A `lcc.kb_records` row. Column order must match the SELECT below.
type KbRecordRow = (
    Uuid,           // id
    Uuid,           // member_id
    String,         // kind
    String,         // title
    String,         // body
    Option<String>, // source
    Vec<String>,    // tags
    Option<String>, // embedding_id
    String,         // embedding_status
    i32,            // version
    DateTime<Utc>,  // created_at
    DateTime<Utc>,  // updated_at
);

/// The last row of a page, from which the next page's cursor is built.
type CursorKey = (DateTime<Utc>, Uuid);

/// Encode a `(created_at, id)` keyset position as an opaque cursor.
pub fn encode_cursor(created_at: DateTime<Utc>, id: Uuid) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(format!("{}|{id}", created_at.to_rfc3339()))
}

/// Decode a cursor produced by [`encode_cursor`].
///
/// A malformed cursor is an error rather than a silent "start from the
/// beginning": returning page one again would look like a working empty page
/// and hide the bug from whoever called it.
fn decode_cursor(cursor: &str) -> Result<CursorKey, Error> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let raw = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|e| Error::Validation(format!("malformed cursor: {e}")))?;
    let raw =
        String::from_utf8(raw).map_err(|e| Error::Validation(format!("malformed cursor: {e}")))?;
    let (ts, id) = raw
        .split_once('|')
        .ok_or_else(|| Error::Validation("malformed cursor".into()))?;
    let created_at = DateTime::parse_from_rfc3339(ts)
        .map_err(|e| Error::Validation(format!("malformed cursor timestamp: {e}")))?
        .with_timezone(&Utc);
    let id = id
        .parse::<Uuid>()
        .map_err(|e| Error::Validation(format!("malformed cursor id: {e}")))?;
    Ok((created_at, id))
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List a member's KB records, newest first.
    ///
    /// `cursor` is the opaque token from a previous page: it encodes the
    /// `(created_at, id)` of the last row seen and the query resumes strictly
    /// after it. Previously the parameter was accepted and dropped, so every
    /// page came back identical and a client scrolling the list would loop on
    /// the same records forever. `id` is part of the key because `created_at`
    /// alone is not unique, and without a tiebreaker the keyset comparison
    /// can skip or repeat rows that share a timestamp.
    pub async fn list(
        &self,
        member_id: Uuid,
        kind: Option<KbKind>,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Vec<KbRecord>, Error> {
        let keyset = cursor.as_deref().map(decode_cursor).transpose()?;

        let rows: Vec<KbRecordRow> = if let Some(kind) = kind {
            sqlx::query_as(
                r#"
                SELECT id, member_id, kind::TEXT, title, body, source, tags,
                       embedding_id, embedding_status::TEXT, version,
                       created_at, updated_at
                FROM lcc.kb_records
                WHERE member_id = $1 AND kind = $2::text
                  AND ($3::timestamptz IS NULL OR (created_at, id) < ($3::timestamptz, $4::uuid))
                ORDER BY created_at DESC, id DESC
                LIMIT $5
                "#,
            )
            .bind(member_id)
            .bind(kind.as_str())
            .bind(keyset.as_ref().map(|(ts, _)| *ts))
            .bind(keyset.as_ref().map(|(_, id)| *id))
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
                  AND ($2::timestamptz IS NULL OR (created_at, id) < ($2::timestamptz, $3::uuid))
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(member_id)
            .bind(keyset.as_ref().map(|(ts, _)| *ts))
            .bind(keyset.as_ref().map(|(_, id)| *id))
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(map_row).collect()
    }

    pub async fn get(&self, member_id: Uuid, id: Uuid) -> Result<KbRecord, Error> {
        let row: KbRecordRow = sqlx::query_as(
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
        map_row(row)
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

    /// Set the embedding status of one record.
    ///
    /// `member_id` is part of the `WHERE` clause, not just the caller's
    /// identity: this is a write, and `lcc.kb_records` has no row-level
    /// security enabled (only one table in the schema does), so scoping by
    /// `id` alone would let any authenticated member write to any other
    /// member's record. A record belonging to someone else now reads as
    /// `NotFound` rather than silently succeeding.
    pub async fn set_embedding_status(
        &self,
        member_id: Uuid,
        id: Uuid,
        status: EmbeddingStatus,
        embedding_id: Option<&str>,
    ) -> Result<(), Error> {
        let res = sqlx::query(
            r#"
            UPDATE lcc.kb_records
            SET embedding_status = $3::text, embedding_id = $4, updated_at = NOW()
            WHERE id = $1 AND member_id = $2
            "#,
        )
        .bind(id)
        .bind(member_id)
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

fn map_row(row: KbRecordRow) -> Result<KbRecord, Error> {
    use serde_json::Value;
    let kind = serde_json::from_value::<KbKind>(Value::String(row.2.clone()))
        .map_err(|e| Error::Internal(format!("kind parse: {e}")))?;
    let embedding_status = serde_json::from_value::<EmbeddingStatus>(Value::String(row.8.clone()))
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
