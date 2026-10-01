//! Canonical REST DTOs for the admin/compliance routes.
//!
//! These mirror `lcc-api-canonical.yaml` paths under
//! `/api/v1/admin/compliance/...`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConfigVersionsRestResponse {
    pub items: Vec<ComplianceConfigVersionDto>,
    pub active_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfigVersionDto {
    pub id: Uuid,
    pub version: String,
    pub status: String, // 'draft' | 'proposed' | 'active' | 'retired'
    pub two_reviewer_signed_by: Vec<String>,
    pub activated_at: Option<DateTime<Utc>>,
    pub previous_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfigVersionRestRequest {
    pub reviewer_user_id: String,
    pub approve: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateConfigVersionRestRequest {
    pub reviewer_a_user_id: String,
    pub reviewer_a_signature: String,
    pub reviewer_b_user_id: String,
    pub reviewer_b_signature: String,
    #[serde(default)]
    pub comment: Option<String>,
}
