// The workspace denies `clippy::unwrap_used`, `clippy::expect_used` and
// `clippy::panic` to keep production code free of panicking shortcuts.
// `cargo clippy --all-targets` — which CI runs — also lints integration-test
// targets, and in a test those are the assertion mechanism: a contract path
// that is not covered MUST fail the build loudly. Exempt this file rather than
// rewriting every assertion into a `match`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Contract-conformance test for the api-gateway routing table.
//!
//! The gateway's routes used to be written directly into
//! `http/router.rs`, so nothing checked them against the contract. That is how
//! `GET /metrics` could be declared in the canonical OpenAPI *and* recorded as
//! live in `docs/endpoint_contract_matrix.md` while never being registered.
//!
//! The routing table is now data (`lcc_api_gateway::routes::ROUTES`). These
//! tests assert that table against `Contract/openapi/lcc-api-canonical.yaml` so
//! the two cannot drift apart silently again.

use std::fs;
use std::path::PathBuf;

use lcc_api_gateway::routes::{self, RouteKind};

fn contract_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let mut dir = PathBuf::from(manifest_dir.as_str());
    loop {
        let candidate = dir.join("Contract");
        if candidate.join("openapi").is_dir() {
            return candidate;
        }
        if !dir.pop() {
            panic!("could not locate Contract/ from {manifest_dir}");
        }
    }
}

fn openapi_source() -> String {
    fs::read_to_string(contract_root().join("openapi/lcc-api-canonical.yaml"))
        .unwrap_or_else(|e| panic!("read canonical openapi: {e}"))
}

/// The path component of `servers[0].url` — the contract expresses the
/// namespace via the server base, not by inlining `/api/v1` in every path.
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
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if let Some(url) = line.trim().strip_prefix("- url:") {
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

/// Paths the gateway serves at the host root rather than under the namespace.
const INFRA_PATHS: [&str; 3] = ["/healthz", "/readyz", "/metrics"];

/// Every canonical path, resolved against the server base.
fn canonical_paths(openapi: &str) -> Vec<String> {
    let base = server_base_path(openapi);
    openapi
        .lines()
        .filter_map(|l| {
            let t = l.trim_start();
            if t.starts_with('/') && t.contains(':') && l.starts_with("  /") {
                let p = t.split(':').next().unwrap_or("").trim().to_string();
                Some(if base.is_empty() || INFRA_PATHS.contains(&p.as_str()) {
                    p
                } else {
                    format!("{base}{p}")
                })
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn gateway_covers_every_canonical_path() {
    let openapi = openapi_source();
    assert_eq!(
        server_base_path(&openapi),
        "/api/v1",
        "canonical contract must declare the /api/v1 namespace via servers[].url"
    );

    let canon = canonical_paths(&openapi);
    assert_eq!(canon.len(), 70, "canonical contract path count changed");

    let uncovered: Vec<&String> = canon
        .iter()
        .filter(|c| {
            !routes::ROUTES
                .iter()
                .any(|r| routes::pattern_matches(r.pattern, c))
        })
        .collect();

    assert!(
        uncovered.is_empty(),
        "{} canonical path(s) are not matched by any gateway route: {uncovered:?}\n\
         Every contract path must be reachable, or the frontend's call 404s at the gateway.",
        uncovered.len(),
    );
}

#[test]
fn every_infra_path_declared_by_the_contract_is_registered() {
    let openapi = openapi_source();
    let canon = canonical_paths(&openapi);

    // The three infra operations are declared `security: []` in the contract,
    // so the gateway must serve them in-process rather than proxying.
    for p in INFRA_PATHS {
        assert!(
            canon.iter().any(|c| c == p),
            "contract no longer declares {p} — update this test"
        );
        let registered = routes::ROUTES
            .iter()
            .any(|r| r.pattern == p && r.kind == RouteKind::Infra);
        assert!(
            registered,
            "{p} is declared by the canonical contract but is not registered as an infra route"
        );
    }
}

#[test]
fn route_patterns_are_unique() {
    let mut seen: Vec<&str> = routes::ROUTES.iter().map(|r| r.pattern).collect();
    seen.sort_unstable();
    let before = seen.len();
    seen.dedup();
    assert_eq!(
        before,
        seen.len(),
        "duplicate route patterns in the routing table would shadow each other: {seen:?}"
    );
}

#[test]
fn every_route_lives_in_the_canonical_namespace() {
    for r in routes::ROUTES {
        let ok = INFRA_PATHS.contains(&r.pattern)
            || r.pattern == "/api/v1/admin/governor/evaluate" // documented legacy
            || r.pattern.starts_with("/api/v1/");
        assert!(
            ok,
            "route {} is outside the canonical namespace (or the documented legacy path)",
            r.pattern
        );
    }
}

#[test]
fn member_literal_routes_precede_the_parametrized_catch_all() {
    // `/api/v1/members/me` must be registered before `/members/:member_id`,
    // otherwise `me` is captured as a member id and every self-service call
    // 404s or proxies to the wrong upstream.
    let me = routes::ROUTES
        .iter()
        .position(|r| r.pattern == "/api/v1/members/me")
        .expect("missing /api/v1/members/me");
    let param = routes::ROUTES
        .iter()
        .position(|r| r.pattern == "/api/v1/members/:member_id")
        .expect("missing /api/v1/members/:member_id");
    let catch_all = routes::ROUTES
        .iter()
        .position(|r| r.pattern == "/api/v1/members/:member_id/*path")
        .expect("missing /api/v1/members/:member_id/*path");

    assert!(me < param, "members/me must precede members/:member_id");
    assert!(
        param < catch_all,
        "members/:member_id must precede the catch-all"
    );
}

#[test]
fn pattern_matching_agrees_with_axum_semantics() {
    // Spot-check the matcher the conformance test relies on.
    assert!(routes::pattern_matches(
        "/api/v1/members/:member_id/*path",
        "/api/v1/members/abc/content/compose"
    ));
    assert!(routes::pattern_matches(
        "/api/v1/members/me",
        "/api/v1/members/me"
    ));
    assert!(!routes::pattern_matches(
        "/api/v1/members/me",
        "/api/v1/members/abc"
    ));
    assert!(routes::pattern_matches(
        "/api/v1/content/*path",
        "/api/v1/content/items"
    ));
    assert!(!routes::pattern_matches(
        "/api/v1/content/*path",
        "/api/v1/analytics/content"
    ));
    // A wildcard needs at least one segment.
    assert!(!routes::pattern_matches(
        "/api/v1/content/*path",
        "/api/v1/content"
    ));
}
