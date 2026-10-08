//! Audit-svc repository — read-only queries against lcc_audit.events.

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::AuditEventRow;
use crate::error::Error;

#[derive(Clone)]
pub struct PgRepository {
    pool: PgPool,
}

/// A `lcc_audit.events` row as it comes back from the filtered listing query.
///
/// Named `StoredAuditRow` to stay clear of the `AuditEvent` domain struct the
/// mapper builds from it.
type StoredAuditRow = (
    i64,           // id -- BIGSERIAL in lcc_audit.events (0011)
    String,        // action        (the event's name)
    Option<Uuid>,  // member_id
    String,        // actor         ('system:<svc>' for service events, 0011)
    DateTime<Utc>, // occurred_at
    Option<Uuid>,  // event_id
    JsonValue,     // metadata      (the event payload)
);

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        member_id: Uuid,
        event_name: Option<&str>,
        producer_service: Option<&str>,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        limit: i64,
    ) -> Result<Vec<AuditEventRow>, Error> {
        // We use a dynamic builder pattern. To keep it static-only and
        // safe, we filter the event_name and producer_service via
        // additional WHERE clauses when present. Sqlx requires the full
        // SQL to be known at compile time, so we conditionally bind.
        let rows: Vec<StoredAuditRow> = sqlx::query_as(
            // The real audit table (lcc_audit.events, migration 0011) names
            // these `action` / `actor` / `metadata` / `event_id`. Mapping:
            //   event_name       -> action    the name of the event
            //   producer_service -> actor     'system:<svc>' for service events
            //   payload          -> metadata  the event body
            //   trace_id (uuid)  -> event_id  the correlation id
            r#"SELECT id, action, member_id, actor,
                      occurred_at, event_id, metadata
               FROM lcc_audit.events
               WHERE member_id = $1
                 AND ($2::TEXT IS NULL OR action = $2)
                 AND ($3::TEXT IS NULL OR actor = $3)
                 AND ($4::DATE IS NULL OR occurred_at::DATE >= $4)
                 AND ($5::DATE IS NULL OR occurred_at::DATE <= $5)
               ORDER BY occurred_at DESC
               LIMIT $6"#,
        )
        .bind(member_id)
        .bind(event_name)
        .bind(producer_service)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, event_name, member_id, producer_service, occurred_at, trace_id, payload)| {
                    AuditEventRow {
                        id,
                        event_name,
                        member_id,
                        producer_service,
                        occurred_at,
                        trace_id,
                        payload,
                    }
                },
            )
            .collect())
    }

    pub async fn get(&self, id: i64) -> Result<AuditEventRow, Error> {
        // Same column mapping as the listing query: action/actor/metadata/
        // event_id, and `id` is a BIGSERIAL.
        let row: StoredAuditRow = sqlx::query_as(
            r#"SELECT id, action, member_id, actor,
                          occurred_at, event_id, metadata
                   FROM lcc_audit.events WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(format!("audit {id}")),
            other => Error::Internal(format!("get audit: {other}")),
        })?;
        Ok(AuditEventRow {
            id: row.0,
            event_name: row.1,
            member_id: row.2,
            producer_service: row.3,
            occurred_at: row.4,
            trace_id: row.5,
            payload: row.6,
        })
    }
}

#[allow(dead_code)]
fn _t(_: DateTime<Utc>) {}
