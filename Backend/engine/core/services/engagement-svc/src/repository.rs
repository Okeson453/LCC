//! Engagement repository.
//!
//! Every statement here is written against the live schema and is kept as a
//! plain string literal (no `format!`) so `tools/extract_sql.py` can PREPARE
//! each one against a real database rather than skipping it as interpolated.
//! The column list is therefore repeated per query; `task_columns_agree` in
//! `tests/repository_contract.rs` fails if the copies ever drift apart.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ActionType, EngagementTask, InboundKind, InboxMessage, TaskStatus};
use crate::error::Error;

/// Longest inbox preview rendered. The contract calls the field `preview` and
/// the proto calls it `body_preview`; `body` itself is the full message, and
/// `LEFT(body, integer)` is bound as a parameter so Postgres resolves the
/// `left(text, integer)` overload.
const PREVIEW_LEN: i32 = 280;

/// The canonical `EngagementTask` row.
///
/// `priority_score` arrives as `DOUBLE PRECISION` and `version` as `INT4`
/// because the column types are `NUMERIC(5,4)` and `BIGINT`; sqlx refuses to
/// decode those into `f64`/`i32`, and casting in SQL keeps the conversion
/// explicit instead of leaving it to a driver's guess.
type EngagementTaskRow = (
    Uuid,                  // id
    Uuid,                  // member_id
    Option<Uuid>,          // contact_id
    Option<String>,        // target_post_id
    String,                // action_type
    String,                // status
    Option<f64>,           // priority_score (NUMERIC(5,4) -> float8)
    Option<DateTime<Utc>>, // due_at
    Option<String>,        // draft_body
    Option<DateTime<Utc>>, // completed_at
    i32,                   // version (BIGINT -> int4)
    DateTime<Utc>,         // created_at
);

/// One `lcc.inbound_messages` row, shaped as the canonical `InboxItem`.
type InboxMessageRow = (
    Uuid,           // id
    Option<Uuid>,   // contact_id
    String,         // kind (lcc.inbound_kind)::TEXT
    String,         // LEFT(body, n) AS preview
    bool,           // read_at IS NULL AS unread
    Option<String>, // thread_id
    DateTime<Utc>,  // received_at
);

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// `GET /api/v1/engagement/queue`.
    ///
    /// Ordering is one definition used by both branches: a ritual queue is read
    /// in the order it must be worked, so the soonest `due_at` leads, the
    /// highest `priority_score` breaks a tie, and `created_at DESC, id` makes
    /// the page deterministic. The previous version ordered one branch by
    /// `priority_score DESC, due_at` and the other by `created_at DESC`, so the
    /// same task moved depending on whether a status filter was supplied.
    pub async fn list_tasks(
        &self,
        member_id: Uuid,
        status: Option<TaskStatus>,
        limit: i64,
    ) -> Result<Vec<EngagementTask>, Error> {
        let rows: Vec<EngagementTaskRow> = if let Some(s) = status {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, target_post_id,
                          action_type, status,
                          priority_score::DOUBLE PRECISION AS priority_score,
                          due_at, draft_body, completed_at,
                          version::INT4 AS version, created_at
                   FROM lcc.engagement_replies
                   WHERE member_id = $1 AND status = $2
                   ORDER BY due_at ASC NULLS LAST,
                            priority_score DESC NULLS LAST,
                            created_at DESC,
                            id
                   LIMIT $3"#,
            )
            .bind(member_id)
            .bind(s.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"SELECT id, member_id, contact_id, target_post_id,
                          action_type, status,
                          priority_score::DOUBLE PRECISION AS priority_score,
                          due_at, draft_body, completed_at,
                          version::INT4 AS version, created_at
                   FROM lcc.engagement_replies
                   WHERE member_id = $1
                   ORDER BY due_at ASC NULLS LAST,
                            priority_score DESC NULLS LAST,
                            created_at DESC,
                            id
                   LIMIT $2"#,
            )
            .bind(member_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(row_to_task).collect()
    }

    /// Read one task, scoped to its owner.
    ///
    /// Used to return the stored row after a mutation. The previous version
    /// synthesised a response object in the service layer, filling
    /// `contact_id`, `action_type`, `created_at` and `updated_at` with `None`,
    /// a hard-coded `ActionType::Reply` and `Utc::now()` — it answered 200 with
    /// values the database never held.
    ///
    /// The `member_id` predicate is part of the WHERE clause, so a caller
    /// cannot read another member's task by id; the service surfaces an empty
    /// result as 404, which is the required non-oracle outcome.
    pub async fn get_task(
        &self,
        member_id: Uuid,
        id: Uuid,
    ) -> Result<Option<EngagementTask>, Error> {
        let row: Option<EngagementTaskRow> = sqlx::query_as(
            r#"SELECT id, member_id, contact_id, target_post_id,
                      action_type, status,
                      priority_score::DOUBLE PRECISION AS priority_score,
                      due_at, draft_body, completed_at,
                      version::INT4 AS version, created_at
               FROM lcc.engagement_replies
               WHERE member_id = $1 AND id = $2"#,
        )
        .bind(member_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_task).transpose()
    }

    /// `POST /api/v1/engagement/tasks`.
    ///
    /// `action_type` and `draft_body` are written explicitly. The previous
    /// version named four columns that do not exist (`contact_id`,
    /// `action_type`, `draft`, `draft_pins`) and omitted `updated_at` and
    /// `version`, which do. `inbound_message_id` and `content_item_id` are
    /// intentionally left NULL: this endpoint creates a fresh unit of work and
    /// nothing in the request identifies an inbound message or a content item.
    pub async fn insert_task(&self, t: &EngagementTask) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO lcc.engagement_replies
                  (id, member_id, contact_id, target_post_id, action_type,
                   status, priority_score, due_at, draft_body, completed_at,
                   version, created_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(t.id)
        .bind(t.member_id)
        .bind(t.contact_id)
        .bind(&t.target_post_id)
        .bind(t.action_type.as_str())
        .bind(t.status.as_str())
        .bind(t.priority_score)
        .bind(t.due_at)
        .bind(&t.draft_body)
        .bind(t.completed_at)
        .bind(i64::from(t.version))
        .bind(t.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// `PATCH /api/v1/engagement/tasks/{id}/draft`.
    ///
    /// `draft_pins` is gone: no column, no contract field, no design field.
    /// `updated_at` is gone for the same reason — `lcc.engagement_replies` has no
    /// such column and no touch trigger (unlike `sequence_steps`, which 0018
    /// gave one), and `version` is what carries the change.
    ///
    /// Returns `None` when the row is absent, is owned by another member, or
    /// its `version` does not match. Those three are deliberately
    /// indistinguishable here, so this statement cannot be used to probe for
    /// another member's task; the service disambiguates with a separate,
    /// equally `member_id`-scoped existence check and answers 404 or 409.
    pub async fn update_task(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
        draft_body: Option<&str>,
        status: Option<TaskStatus>,
    ) -> Result<Option<i32>, Error> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET draft_body = COALESCE($4, draft_body),
                   status = COALESCE($5, status),
                   version = version + 1
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version::INT4 AS version"#,
        )
        .bind(member_id)
        .bind(id)
        .bind(i64::from(expected_version))
        .bind(draft_body)
        .bind(status.map(|s| s.as_str()))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// `POST /api/v1/engagement/tasks/{id}/complete`.
    ///
    /// `status = 'sent'`, not the previous `'completed'`: `completed` is in
    /// neither design §11.12's CHECK list nor the contract enum nor the proto
    /// enum, so writing it produced a row no canonical client could interpret.
    pub async fn complete_task(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
    ) -> Result<Option<i32>, Error> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET status = 'sent', completed_at = NOW(),
                   version = version + 1
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version::INT4 AS version"#,
        )
        .bind(member_id)
        .bind(id)
        .bind(i64::from(expected_version))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// `POST /api/v1/engagement/tasks/{id}/dismiss`.
    ///
    /// `status = 'dismissed'`, matching the contract enum and the proto's
    /// `ENGAGEMENT_STATUS_DISMISSED` — not the previous `'skipped'`.
    pub async fn dismiss_task(
        &self,
        member_id: Uuid,
        id: Uuid,
        expected_version: i32,
    ) -> Result<Option<i32>, Error> {
        let row: Option<(i32,)> = sqlx::query_as(
            r#"UPDATE lcc.engagement_replies
               SET status = 'dismissed', version = version + 1
               WHERE member_id = $1 AND id = $2 AND version = $3
               RETURNING version::INT4 AS version"#,
        )
        .bind(member_id)
        .bind(id)
        .bind(i64::from(expected_version))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// `GET /api/v1/engagement/inbox`.
    ///
    /// Repointed from the non-existent `lcc.engagement_inbox` to
    /// `lcc.inbound_messages`, which is what
    /// `Contract/docs/endpoint_contract_matrix.md:341` records
    /// ("`InboxItem` | `lcc.inbound_messages` | 0007") and what design §11
    /// stores inbound traffic in.
    ///
    /// Three of the seven projected values are derived rather than selected:
    ///   * `preview` <- `LEFT(body, n)`  (the proto's `body_preview`)
    ///   * `unread`  <- `read_at IS NULL` (the schema has no boolean)
    ///   * `kind`    <- `kind::TEXT`, verbatim, so no cast can change it
    ///
    /// `subject` is gone: `lcc.inbound_messages` has no subject column, and
    /// neither the proto's `InboxItem` nor design §11 declares one.
    /// `thread_id` is added because both the contract and the proto carry it and
    /// the column is real.
    ///
    /// Archived messages are excluded. `is_archived BOOLEAN NOT NULL DEFAULT
    /// FALSE` (0007:18) exists precisely to take a message out of the working
    /// set, so listing one under a 200 would return something the member has
    /// put away.
    pub async fn inbox(
        &self,
        member_id: Uuid,
        limit: i64,
        unread_only: bool,
    ) -> Result<Vec<InboxMessage>, Error> {
        let rows: Vec<InboxMessageRow> = if unread_only {
            sqlx::query_as(
                r#"SELECT id, contact_id, kind::TEXT AS kind,
                          LEFT(body, $2) AS preview,
                          (read_at IS NULL) AS unread,
                          thread_id, received_at
                   FROM lcc.inbound_messages
                   WHERE member_id = $1 AND read_at IS NULL AND is_archived = FALSE
                   ORDER BY received_at DESC, id DESC
                   LIMIT $3"#,
            )
            .bind(member_id)
            .bind(PREVIEW_LEN)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"SELECT id, contact_id, kind::TEXT AS kind,
                          LEFT(body, $2) AS preview,
                          (read_at IS NULL) AS unread,
                          thread_id, received_at
                   FROM lcc.inbound_messages
                   WHERE member_id = $1 AND is_archived = FALSE
                   ORDER BY received_at DESC, id DESC
                   LIMIT $3"#,
            )
            .bind(member_id)
            .bind(PREVIEW_LEN)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(row_to_inbox).collect()
    }

    /// `POST /api/v1/engagement/inbox/{id}/read`.
    ///
    /// Repointed to `lcc.inbound_messages.read_at`, the real column. Returns
    /// `false` when no row matched, and the handler still answers 204 in that
    /// case: a member marking a message they cannot see must not be able to
    /// learn whether it exists.
    pub async fn mark_inbox_read(&self, member_id: Uuid, id: Uuid) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"UPDATE lcc.inbound_messages
               SET read_at = NOW()
               WHERE member_id = $1 AND id = $2 AND read_at IS NULL"#,
        )
        .bind(member_id)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Map a stored row to the domain type.
///
/// The two enum columns are parsed with an explicit `FromStr` rather than
/// through serde. A value outside the vocabulary is a data defect, and it must
/// surface as an error naming the offending value; serde's "unknown variant"
/// message does not say which value was seen.
fn row_to_task(row: EngagementTaskRow) -> Result<EngagementTask, Error> {
    let (
        id,
        member_id,
        contact_id,
        target_post_id,
        action_type,
        status,
        priority_score,
        due_at,
        draft_body,
        completed_at,
        version,
        created_at,
    ) = row;

    let action_type: ActionType = action_type.parse().map_err(|v| {
        Error::Internal(format!(
            "engagement_replies.action_type: unknown value {v:?}"
        ))
    })?;
    let status: TaskStatus = status
        .parse()
        .map_err(|v| Error::Internal(format!("engagement_replies.status: unknown value {v:?}")))?;

    Ok(EngagementTask {
        id,
        member_id,
        contact_id,
        target_post_id,
        action_type,
        status,
        priority_score,
        due_at,
        draft_body,
        completed_at,
        version,
        created_at,
    })
}

fn row_to_inbox(row: InboxMessageRow) -> Result<InboxMessage, Error> {
    let (id, contact_id, kind, preview, unread, thread_id, received_at) = row;
    let kind: InboundKind = kind
        .parse()
        .map_err(|v| Error::Internal(format!("inbound_messages.kind: unknown value {v:?}")))?;
    Ok(InboxMessage {
        id,
        contact_id,
        kind,
        preview,
        unread,
        thread_id,
        received_at,
    })
}
