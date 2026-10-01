//! Audit-svc domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRow {
    pub id: Uuid,
    pub event_name: String,
    pub member_id: Option<Uuid>,
    pub producer_service: String,
    pub occurred_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub payload: serde_json::Value,
}
