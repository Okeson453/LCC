//! Audit-svc domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRow {
    /// `lcc_audit.events.id` is a BIGSERIAL, not a uuid.
    pub id: i64,
    /// The event's name. Stored in the `action` column.
    pub event_name: String,
    pub member_id: Option<Uuid>,
    /// Which service emitted the event. Stored in the `actor` column, which
    /// the schema documents as 'system:<svc>' for service-originated rows.
    pub producer_service: String,
    pub occurred_at: DateTime<Utc>,
    /// Correlation id, stored in `event_id` (nullable in the real schema).
    pub trace_id: Option<Uuid>,
    /// The event body, stored in the `metadata` JSONB column.
    pub payload: serde_json::Value,
}
