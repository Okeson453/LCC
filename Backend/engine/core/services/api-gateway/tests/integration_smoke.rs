//! Smoke tests for api-gateway.
//!
//! api-gateway is the stateless public edge: it terminates HTTP, verifies
//! JWTs, rate-limits and reverse-proxies. It holds no database or Redis pool
//! of its own, so unlike the domain services there is no `database_url` /
//! `redis_url` to assert — what it must get right is that every upstream it
//! proxies to is configured and on the right port (F-AUDIT-37).

use lcc_api_gateway::config::Config;

#[test]
fn config_defaults_load() {
    let cfg = Config::default();
    assert!(cfg.http_port > 0);
}

#[test]
fn service_name_correct() {
    let cfg = Config::default();
    assert_eq!(cfg.service_name, "api-gateway");
}

#[test]
fn audit_svc_url_set() {
    let cfg = Config::default();
    assert!(!cfg.audit_svc_url.is_empty());
}

#[test]
fn compliance_governor_url_set() {
    let cfg = Config::default();
    assert!(!cfg.compliance_governor_url.is_empty());
}

#[test]
fn every_upstream_is_configured() {
    let cfg = Config::default();
    for url in [
        &cfg.orchestrator_url,
        &cfg.identity_svc_url,
        &cfg.profile_svc_url,
        &cfg.content_svc_url,
        &cfg.engagement_svc_url,
        &cfg.network_svc_url,
        &cfg.opportunity_svc_url,
        &cfg.outreach_svc_url,
        &cfg.analytics_svc_url,
        &cfg.approval_svc_url,
        &cfg.kb_svc_url,
        &cfg.realtime_svc_url,
    ] {
        assert!(!url.is_empty());
    }
}

#[test]
fn upstreams_are_not_all_on_the_gateway_port() {
    // F-AUDIT-37: every upstream was pinned to :8080, which only api-gateway
    // binds, so each proxy route was a connection-refused -> 502.
    let cfg = Config::default();
    for url in [
        &cfg.orchestrator_url,
        &cfg.identity_svc_url,
        &cfg.content_svc_url,
        &cfg.analytics_svc_url,
    ] {
        assert!(
            !url.ends_with(":8080"),
            "upstream {url} points at the gateway's own port"
        );
    }
}
