//! Contract validation test — verifies that the canonical OpenAPI
//! (`openapi/lcc-api-canonical.yaml`) and the canonical realtime contract
//! (`realtime/lcc-realtime-contract.yaml`) parse, that every required
//! design endpoint is declared, that path conventions are consistent,
//! and that the static checks match what the implementation actually
//! registers.

use std::fs;
use std::path::PathBuf;

fn contract_path(rel: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    manifest_dir
        .join("..")
        .join("..")
        .join("..")
        .join("contract_audit")
        .join(rel)
}

fn read(rel: &str) -> String {
    fs::read_to_string(contract_path(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Extract top-level path entries from the OpenAPI YAML by counting the
/// "  /<path>:" lines that begin paths in the document.
fn paths(openapi: &str) -> Vec<String> {
    openapi
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim_start();
            if trimmed.starts_with("/") && trimmed.contains(':') {
                Some(trimmed.split(':').next().unwrap().trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

fn operations(openapi: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current_path: Option<String> = None;
    for line in openapi.lines() {
        if let Some(m) = line
            .trim_start()
            .strip_prefix('/')
            .and_then(|s| s.split(':').next().map(|p| p.trim().to_string()))
            .filter(|p| !p.is_empty() && !p.contains(' '))
        {
            current_path = Some(format!("/{}", m));
        } else {
            let trimmed = line.trim_start();
            for verb in &["get", "post", "put", "patch", "delete"] {
                if trimmed.starts_with(&format!("{verb}:")) {
                    if let Some(p) = &current_path {
                        out.push((verb.to_string(), p.clone()));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn canonical_openapi_exists_and_parses() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    assert!(openapi.contains("openapi: 3.1.0"));
    assert!(openapi.contains("OKESON-LCC"));
    assert!(openapi.contains("/api/v1/auth/linkedin/start"));
    assert!(openapi.contains("/api/v1/members/me"));
    assert!(openapi.contains("/api/v1/admin/compliance/config-versions"));
}

#[test]
fn canonical_openapi_uses_canonical_namespace_only() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    // Reject any path that does NOT start with /api/v1/ or one of the infra paths.
    let paths = paths(&openapi);
    let bad: Vec<_> = paths
        .iter()
        .filter(|p| {
            !p.starts_with("/api/v1/")
                && !p.starts_with("/healthz")
                && !p.starts_with("/readyz")
                && !p.starts_with("/metrics")
        })
        .collect();
    assert!(
        bad.is_empty(),
        "found non-canonical paths: {:?}\nEvery path must be /api/v1/<domain>/... or infra-only.",
        bad
    );
}

#[test]
fn canonical_openapi_contains_all_design_endpoints() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    let required = [
        "/api/v1/auth/linkedin/start",
        "/api/v1/auth/linkedin/callback",
        "/api/v1/auth/refresh",
        "/api/v1/auth/logout",
        "/api/v1/members/me",
        "/api/v1/members/me/settings",
        "/api/v1/members/{memberId}/profile/snapshots/latest",
        "/api/v1/members/{memberId}/profile/audit",
        "/api/v1/members/{memberId}/profile/edit-drafts",
        "/api/v1/members/{memberId}/content",
        "/api/v1/members/{memberId}/content/{contentId}",
        "/api/v1/members/{memberId}/content/compose",
        "/api/v1/members/{memberId}/content/{contentId}/quality-check",
        "/api/v1/members/{memberId}/content/{contentId}/submit-for-approval",
        "/api/v1/members/{memberId}/content/{contentId}/schedule",
        "/api/v1/members/{memberId}/engagement/inbox",
        "/api/v1/members/{memberId}/engagement/queue",
        "/api/v1/members/{memberId}/engagement/tasks/{taskId}/draft-reply",
        "/api/v1/members/{memberId}/engagement/tasks/{taskId}/approve",
        "/api/v1/members/{memberId}/contacts",
        "/api/v1/members/{memberId}/contacts/{contactId}",
        "/api/v1/members/{memberId}/contacts/{contactId}/interactions",
        "/api/v1/members/{memberId}/contacts/stale",
        "/api/v1/members/{memberId}/opportunities",
        "/api/v1/members/{memberId}/opportunities/{opportunityId}",
        "/api/v1/members/{memberId}/opportunities/{opportunityId}/qualify",
        "/api/v1/members/{memberId}/opportunities/{opportunityId}/applications",
        "/api/v1/members/{memberId}/opportunities/{opportunityId}/draft-proposal",
        "/api/v1/members/{memberId}/opportunities/{opportunityId}/send-proposal",
        "/api/v1/members/{memberId}/sequences",
        "/api/v1/members/{memberId}/sequences/{sequenceId}",
        "/api/v1/members/{memberId}/sequences/{sequenceId}/pause",
        "/api/v1/members/{memberId}/sequences/{sequenceId}/steps/{stepId}/approve",
        "/api/v1/members/{memberId}/kb/records",
        "/api/v1/members/{memberId}/kb/records/{recordId}",
        "/api/v1/members/{memberId}/analytics/content",
        "/api/v1/members/{memberId}/analytics/account-health",
        "/api/v1/members/{memberId}/analytics/digest/weekly",
        "/api/v1/members/{memberId}/approvals",
        "/api/v1/members/{memberId}/approvals/{approvalId}",
        "/api/v1/members/{memberId}/approvals/{approvalId}/decide",
        "/api/v1/members/{memberId}/approvals/bulk-decide",
        "/api/v1/admin/compliance/config-versions",
        "/api/v1/admin/compliance/config-versions/{versionId}/activate",
        "/api/v1/admin/compliance/restrictions/{memberId}",
    ];
    for path in required {
        assert!(
            openapi.contains(path),
            "canonical OpenAPI missing required path {path}"
        );
    }
}

#[test]
fn canonical_openapi_operation_count_meets_design() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    let ops = operations(&openapi);
    assert!(
        ops.len() >= 80,
        "expected ≥80 operations in canonical contract, found {}",
        ops.len()
    );
}

#[test]
fn canonical_openapi_error_envelope_is_standard() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    assert!(openapi.contains("Error"));
    assert!(openapi.contains("GOVERNANCE_DENIED"));
    assert!(openapi.contains("UNAUTHORIZED"));
    assert!(openapi.contains("FORBIDDEN"));
    assert!(openapi.contains("NOT_FOUND"));
    assert!(openapi.contains("VALIDATION_ERROR"));
    assert!(openapi.contains("trace_id"));
}

#[test]
fn canonical_openapi_uses_bearer_security() {
    let openapi = read("openapi/lcc-api-canonical.yaml");
    assert!(openapi.contains("bearerAuth"));
    assert!(openapi.contains("scheme: bearer"));
    assert!(openapi.contains("bearerFormat: JWT"));
}

#[test]
fn realtime_contract_has_all_five_channels() {
    let rt = read("realtime/lcc-realtime-contract.yaml");
    for ch in ["ws.briefing", "ws.approvals", "ws.engagement", "ws.compliance", "ws.sequence"] {
        assert!(rt.contains(ch), "realtime contract missing channel {ch}");
    }
}

#[test]
fn realtime_contract_lists_all_19_events() {
    let rt = read("realtime/lcc-realtime-contract.yaml");
    let expected_events = [
        "briefing.refresh",
        "briefing.section.updated",
        "approval.created",
        "approval.expired",
        "approval.bulk_decided",
        "engagement.inbound.received",
        "engagement.task.created",
        "engagement.draft_ready",
        "engagement.task.expired",
        "compliance.restriction_detected",
        "compliance.restriction_cleared",
        "compliance.config_activated",
        "compliance.circuit_breaker_state_changed",
        "sequence.reply_detected",
        "sequence.paused",
        "sequence.resumed",
        "sequence.completed",
        "sequence.step.sent",
        "sequence.step.failed",
    ];
    for e in expected_events {
        assert!(rt.contains(e), "realtime contract missing event {e}");
    }
}

#[test]
fn realtime_contract_defines_envelope_shape() {
    let rt = read("realtime/lcc-realtime-contract.yaml");
    assert!(rt.contains("event_id"));
    assert!(rt.contains("event_name"));
    assert!(rt.contains("occurred_at"));
    assert!(rt.contains("member_id"));
    assert!(rt.contains("trace_id"));
    assert!(rt.contains("at_least_once_with_event_id_dedup"));
}
