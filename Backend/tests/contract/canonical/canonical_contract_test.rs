//! Contract validation test — verifies that the canonical OpenAPI
//! (`openapi/lcc-api-canonical.yaml`) and the canonical realtime contract
//! (`realtime/lcc-realtime-contract.yaml`) parse, that every required
//! design endpoint is declared, that path conventions are consistent,
//! and that the static checks match what the implementation actually
//! registers.

use std::fs;
use std::path::PathBuf;

/// Resolve the repository's `Contract/` directory.
///
/// The contract lives at `<repo>/Contract/`, but the crate this test is built
/// into sits at `<repo>/Backend/engine/core/services/<svc>/`, so a fixed
/// `../../..` hop landed on `Backend/contract_audit` — a directory that does
/// not exist. Walking up until `Contract/openapi` appears makes the lookup
/// independent of how deep the owning crate is.
fn contract_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let mut dir = PathBuf::from(manifest_dir.as_str());
    loop {
        let candidate = dir.join("Contract");
        if candidate.join("openapi").is_dir() {
            return candidate;
        }
        // Also accept running from inside the Contract dir itself.
        if dir.file_name().map(|n| n == "Contract").unwrap_or(false) {
            return dir;
        }
        if !dir.pop() {
            panic!("could not locate the Contract/ directory by walking up from {manifest_dir}");
        }
    }
}

fn contract_path(rel: &str) -> PathBuf {
    contract_root().join(rel)
}

fn read(rel: &str) -> String {
    fs::read_to_string(contract_path(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// The path component of the OpenAPI `servers[0].url`, e.g. `/api/v1`.
///
/// The canonical contract declares the namespace the idiomatic OpenAPI 3.1
/// way — `servers: [{ url: https://…/api/v1 }]` plus paths relative to that
/// base — rather than inlining `/api/v1` into all 70 path keys. Reading the
/// path keys alone therefore yields `/auth/linkedin/start`, and the earlier
/// version of this test asserted against the inlined form and failed 3/9 even
/// though the contract was correct. Resolving the server base here means the
/// assertions below run against the *effective* URL, which is what the
/// gateway actually routes.
fn server_base_path(openapi: &str) -> String {
    let mut in_servers = false;
    for line in openapi.lines() {
        if line.starts_with("servers:") {
            in_servers = true;
            continue;
        }
        if !in_servers {
            continue;
        }
        // A new top-level key ends the `servers:` block.
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if let Some(url) = line.trim().strip_prefix("- url:") {
            // Take the path component of the URL, if any.
            if let Some(idx) = url.find("://") {
                if let Some(slash) = url[idx + 3..].find('/') {
                    return url[idx + 3 + slash..].trim_end_matches('/').to_string();
                }
            }
            return String::new();
        }
    }
    String::new()
}

/// Paths that the gateway serves at the host root rather than under the
/// versioned namespace (`health::router()` registers these verbatim).
const INFRA_PATHS: [&str; 3] = ["/healthz", "/readyz", "/metrics"];

/// Extract top-level path entries, resolving each against the server base so
/// the result is the effective request path. Infra paths stay at the root.
fn paths(openapi: &str) -> Vec<String> {
    let base = server_base_path(openapi);
    openapi
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim_start();
            if trimmed.starts_with('/') && trimmed.contains(':') {
                let p = trimmed.split(':').next().unwrap().trim().to_string();
                Some(resolve_path(&p, &base))
            } else {
                None
            }
        })
        .collect()
}

fn resolve_path(p: &str, base: &str) -> String {
    if base.is_empty() || INFRA_PATHS.contains(&p) {
        return p.to_string();
    }
    format!("{}{}", base, p)
}

fn operations(openapi: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let base = server_base_path(openapi);
    let mut current_path: Option<String> = None;
    for line in openapi.lines() {
        if let Some(m) = line
            .trim_start()
            .strip_prefix('/')
            .and_then(|s| s.split(':').next().map(|p| p.trim().to_string()))
            .filter(|p| !p.is_empty() && !p.contains(' '))
        {
            current_path = Some(resolve_path(&format!("/{m}"), &base));
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
    // Assert against resolved effective paths, not raw YAML substrings — the
    // contract expresses the namespace via `servers[].url`.
    let paths = paths(&openapi);
    for expected in [
        "/api/v1/auth/linkedin/start",
        "/api/v1/members/me",
        "/api/v1/admin/compliance/config-versions",
    ] {
        assert!(
            paths.iter().any(|p| p == expected),
            "canonical OpenAPI is missing effective path {expected}; resolved paths: {paths:?}"
        );
    }
    // The server base must actually carry the version prefix, otherwise every
    // resolved path above would collapse onto the host root.
    assert_eq!(server_base_path(&openapi), "/api/v1");
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
    let declared = paths(&openapi);
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|path| !declared.iter().any(|p| p == path))
        .collect();
    assert!(
        missing.is_empty(),
        "canonical OpenAPI missing required paths: {missing:?}"
    );
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
    for ch in [
        "ws.briefing",
        "ws.approvals",
        "ws.engagement",
        "ws.compliance",
        "ws.sequence",
    ] {
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
