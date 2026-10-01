//! Smoke tests for identity-svc.

use lcc_identity_svc::config::Config;
use serde_json::json;

#[test]
fn config_defaults_load() {
    let cfg = Config::default();
    assert!(cfg.http_port > 0);
}

#[test]
fn service_name_correct() {
    let cfg = Config::default();
    assert_eq!(cfg.service_name, "identity-svc");
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
fn database_url_set() {
    let cfg = Config::default();
    assert!(!cfg.database_url.is_empty());
}

#[test]
fn redis_url_set() {
    let cfg = Config::default();
    assert!(!cfg.redis_url.is_empty());
}
