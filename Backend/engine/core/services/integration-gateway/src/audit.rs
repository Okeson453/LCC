//! Audit emission for the Integration Gateway.
//!
//! Every executed action emits an `audit_log` row with the outcome, reason,
//! and SHA-256 checksum. Restricted-signal actions emit with outcome='denied'
//! to provide visibility into account-safety events.

use chrono::Utc;
use lcc_audit_client::{AuditClient, AuditEvent, AuditOutcome};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::restriction::RestrictionSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAuditEntry {
    pub action_id: Uuid,
    pub member_id: Uuid,
    pub action_type: String,
    pub track: String,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub restriction_signal: Option<RestrictionSignal>,
    pub duration_ms: u64,
}

pub async fn emit(
    client: &AuditClient,
    entry: IntegrationAuditEntry,
) -> Result<i64, lcc_audit_client::AuditError> {
    let mut event = AuditEvent::new(
        "system:integration-gateway",
        "integration.execute",
        "candidate_action",
    )
    .resource_id(entry.action_id)
    .member_id(entry.member_id)
    .reason(entry.reason.clone().unwrap_or_default())
    .idempotency_key(format!("{}:{}", entry.action_id, Utc::now().timestamp()))
    .outcome(entry.outcome);

    if let Some(sig) = entry.restriction_signal {
        if sig != RestrictionSignal::None {
            event = event.reason(format!(
                "restriction_signal: {} — {}",
                sig.as_str(),
                entry.reason.clone().unwrap_or_default()
            ));
        }
    }

    client.record(event).await
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_entry_serializes() {
        let entry = IntegrationAuditEntry {
            action_id: Uuid::now_v7(),
            member_id: Uuid::now_v7(),
            action_type: "post_publish".into(),
            track: "A".into(),
            outcome: AuditOutcome::Success,
            reason: Some("ugc_post_published".into()),
            restriction_signal: None,
            duration_ms: 230,
        };
        let s = serde_json::to_string(&entry).unwrap();
        assert!(s.contains("post_publish"));
    }
}
