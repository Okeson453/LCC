//! Gateway routing contract test.
//!
//! Verifies that the canonical inbound paths map to the correct upstream
//! service via the proxy module's `UpstreamRegistry::resolve` and
//! `rewrite_for_upstream` methods. This test does not need a live
//! database; it only exercises the routing logic.

#![cfg(test)]

use lcc_api_gateway::config::ApiGatewayConfig;
use lcc_api_gateway::proxy::UpstreamRegistry;
use lcc_api_gateway::state::build_upstream_registry;

fn config() -> ApiGatewayConfig {
    ApiGatewayConfig {
        service_name: "api-gateway-test".into(),
        http_port: 8080,
        environment: "local".into(),
        log_level: "info".into(),
        auth_jwt_audience: "lcc-api".into(),
        auth_jwt_issuer: "lcc-identity-svc".into(),
        auth_jwt_secret: "x".into(),
        rate_limit_per_minute: 60,
        audit_svc_url: "http://audit-svc:8091".into(),
        orchestrator_url: "http://orchestrator:8081".into(),
        identity_svc_url: "http://identity-svc:8090".into(),
        profile_svc_url: "http://profile-svc:8082".into(),
        content_svc_url: "http://content-svc:8083".into(),
        engagement_svc_url: "http://engagement-svc:8084".into(),
        network_svc_url: "http://network-crm-svc:8085".into(),
        opportunity_svc_url: "http://opportunity-svc:8086".into(),
        outreach_svc_url: "http://outreach-svc:8087".into(),
        analytics_svc_url: "http://analytics-svc:8088".into(),
        approval_svc_url: "http://approval-svc:8089".into(),
        compliance_governor_url: "http://compliance-governor:8080".into(),
        kb_svc_url: "http://kb-svc:8092".into(),
        realtime_svc_url: "http://realtime-svc:8093".into(),
    }
}

#[test]
fn registry_resolves_canonical_paths_to_correct_upstreams() {
    let cfg = config();
    let registry = build_upstream_registry(&cfg);

    let cases: &[(&str, &str)] = &[
        ("/api/v1/auth/linkedin/start", "identity-svc"),
        ("/api/v1/auth/refresh", "identity-svc"),
        ("/api/v1/members/me", "identity-svc"),
        ("/api/v1/members/{memberId}/profile/snapshots/latest", "profile-svc"),
        ("/api/v1/members/{memberId}/content", "content-svc"),
        ("/api/v1/members/{memberId}/content/{contentId}/quality-check", "content-svc"),
        ("/api/v1/members/{memberId}/engagement/queue", "engagement-svc"),
        ("/api/v1/members/{memberId}/contacts/stale", "network-crm-svc"),
        ("/api/v1/members/{memberId}/opportunities", "opportunity-svc"),
        ("/api/v1/members/{memberId}/opportunities/{opportunityId}/draft-proposal", "opportunity-svc"),
        ("/api/v1/members/{memberId}/sequences", "outreach-svc"),
        ("/api/v1/members/{memberId}/sequences/{sequenceId}/pause", "outreach-svc"),
        ("/api/v1/members/{memberId}/kb/records", "kb-svc"),
        ("/api/v1/members/{memberId}/analytics/account-health", "analytics-svc"),
        ("/api/v1/members/{memberId}/briefing/today", "orchestrator"),
        ("/api/v1/members/{memberId}/approvals", "approval-svc"),
        ("/api/v1/members/{memberId}/approvals/{approvalId}/decide", "approval-svc"),
        ("/api/v1/members/{memberId}/audit", "audit-svc"),
        ("/api/v1/admin/compliance/config-versions", "compliance-governor"),
        ("/api/v1/admin/compliance/restrictions/{memberId}", "compliance-governor"),
        ("/api/v1/ws/briefing", "realtime-svc"),
        ("/api/v1/ws/sequence", "realtime-svc"),
    ];

    for (path, expected_upstream) in cases {
        let pool = registry
            .resolve(path)
            .unwrap_or_else(|_| panic!("no upstream bound for {path}"));
        assert_eq!(
            pool.upstream_name(),
            *expected_upstream,
            "path {path} should resolve to {expected_upstream}"
        );
    }
}

#[test]
fn registry_rewrites_legacy_upstreams_correctly() {
    let cfg = config();
    let registry = build_upstream_registry(&cfg);

    // identity-svc implements the canonical namespace — no rewrite.
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/members/me", "identity-svc"),
        "/api/v1/members/me"
    );
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/auth/refresh", "identity-svc"),
        "/api/v1/auth/refresh"
    );

    // compliance-governor also implements canonical — no rewrite.
    assert_eq!(
        registry.rewrite_for_upstream(
            "/api/v1/admin/compliance/config-versions",
            "compliance-governor"
        ),
        "/api/v1/admin/compliance/config-versions"
    );

    // content-svc still uses legacy /v1/content_svc/items/...
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/content", "content-svc"),
        "/v1/content_svc"
    );
    assert_eq!(
        registry.rewrite_for_upstream(
            "/api/v1/content/{contentId}/quality-check",
            "content-svc"
        ),
        "/v1/content_svc/{contentId}/quality-check"
    );

    // realtime-svc uses canonical WS paths — no rewrite.
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/ws/briefing", "realtime-svc"),
        "/api/v1/ws/briefing"
    );

    // engagement-svc uses legacy shape.
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/engagement/queue", "engagement-svc"),
        "/v1/engagement_svc/queue"
    );

    // opportunities still legacy.
    assert_eq!(
        registry.rewrite_for_upstream(
            "/api/v1/opportunities/{oppId}/draft-application",
            "opportunity-svc"
        ),
        "/v1/opportunity_svc/{oppId}/draft-application"
    );

    // sequences — outreach-svc.
    assert_eq!(
        registry.rewrite_for_upstream("/api/v1/sequences/{seqId}/pause", "outreach-svc"),
        "/v1/outreach_svc/{seqId}/pause"
    );

    // analytics
    assert_eq!(
        registry.rewrite_for_upstream(
            "/api/v1/analytics/account-health",
            "analytics-svc"
        ),
        "/v1/analytics_svc/account-health"
    );
}

#[test]
fn registry_rejects_unknown_domains() {
    let cfg = config();
    let registry = build_upstream_registry(&cfg);
    let res = registry.resolve("/api/v1/no-such-domain/foo");
    assert!(res.is_err());
}

#[test]
fn registry_accepts_legacy_v1_paths_for_deprecation_window() {
    let cfg = config();
    let registry = build_upstream_registry(&cfg);

    // Legacy /v1/<domain>/... should still resolve during the migration
    // window so older clients keep working.
    let pool = registry.resolve("/v1/content/items").expect("legacy path");
    assert_eq!(pool.upstream_name(), "content-svc");
}
