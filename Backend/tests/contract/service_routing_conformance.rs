// The workspace denies `clippy::unwrap_used` / `clippy::expect_used` and CI
// runs `clippy --all-targets`, which lints integration tests too. Exempt this
// file rather than turning every assertion into a `match`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Every service router must serve the canonical paths, under the canonical
//! HTTP methods.
//!
//! The three layers had drifted apart:
//!
//! - `lcc-api-canonical.yaml` nests the domain surface under
//!   `/api/v1/members/{memberId}/<domain>/…`;
//! - ten domain services still registered the flat `/api/v1/<domain>/…`;
//! - `api-gateway` follows the contract and forwards paths *verbatim*, so
//!   every one of those flat routes sat behind a gateway that never routed
//!   them — reachable directly, unreachable in production.
//!
//! `gateway_contract_conformance.rs` proves the gateway covers the contract.
//! This proves the other end of that hop: that the thing the gateway forwards
//! to actually listens on the forwarded path.
//!
//! The check is static — it parses each service's `.route("…")` literals out of
//! its source. That is deliberate: `build_router` needs an `AppState`, which
//! needs Postgres, so a live introspection test would only run where the
//! integration suite already runs and would be skipped in the fast gate. The
//! route table is written by hand, so the literal is the contract.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Services whose router is owned by a different namespace, and why they are
/// exempt from the member-scoped rule.
fn exempt(service: &str) -> Option<&'static str> {
    match service {
        // Identity owns the OAuth handshake and `/members/me` itself.
        "identity-svc" => Some("auth + members"),
        // Realtime is transport-only: `/ws/*` and `/sse/*`, no member segment.
        "realtime-svc" => Some("ws/sse transports"),
        // Admin routes are operator-scoped, not member-scoped.
        "compliance-governor" => Some("admin surface"),
        // The gateway is the router under test elsewhere; it has no upstream.
        "api-gateway" | "integration-gateway" => Some("gateway itself"),
        // Bounded-context scaffolding with no HTTP surface yet.
        "orchestrator" => Some("briefing is already member-scoped"),
        _ => None,
    }
}

/// Services still on the flat namespace, with the count of routes to move.
///
/// This list is a migration ledger, not a tolerance: `PENDING_MIGRATION` may
/// only shrink. `pending_namespace_migration_is_exactly_the_ledger` fails if a
/// service is added to it or a route is removed from it, so finishing a service
/// is a deliberate edit here rather than a silent improvement, and nothing new
/// can quietly join the flat namespace.
const PENDING_MIGRATION: &[(&str, usize)] = &[
    ("analytics-svc", 2),
    ("audit-svc", 2),
    ("engagement-svc", 7),
    ("kb-svc", 4),
    ("network-crm-svc", 6),
    ("opportunity-svc", 5),
    ("outreach-svc", 7),
    ("profile-svc", 4),
];

/// Endpoints a service implements that the canonical contract does not
/// declare — i.e. real, callable API surface with no contract entry.
///
/// These are candidate *contract omissions*, not service drift, so they are
/// listed rather than deleted: removing `POST /approvals` would delete a
/// working capability, and adding it to the contract is a decision about the
/// published API, not a mechanical fix. Each entry is `(service, path,
/// extra methods)`, and `declared_surface_gaps_are_exactly_the_ledger` fails
/// if the set changes, so the list has to be justified rather than grown.
const CONTRACT_OMISSIONS: &[(&str, &str, &[&str])] = &[(
    "approval-svc",
    "/api/v1/members/:member_id/approvals",
    &["POST"],
)];

fn is_omission(svc: &str, path: &str, method: &str) -> bool {
    CONTRACT_OMISSIONS
        .iter()
        .any(|(s, p, ms)| *s == svc && *p == path && ms.contains(&method))
}

fn is_pending(service: &str) -> bool {
    PENDING_MIGRATION.iter().any(|(s, _)| *s == service)
}

fn repo_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let mut dir = PathBuf::from(manifest_dir.as_str());
    loop {
        if dir
            .join("Contract/openapi/lcc-api-canonical.yaml")
            .is_file()
        {
            return dir;
        }
        if !dir.pop() {
            panic!("could not locate repo root from {manifest_dir}");
        }
    }
}

/// `POST /api/v1/…` / `get(x).post(y)` / `axum::routing::patch(x)` → methods.
fn methods_for(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    for verb in ["get", "post", "put", "patch", "delete"] {
        if regex_lite_contains(expr, verb) {
            out.push(verb.to_uppercase());
        }
    }
    out
}

/// Dependency-free substring regex: the route expressions only ever contain
/// bare verb identifiers, so a word-boundary check is enough and avoids
/// pulling `regex` into the test.
fn regex_lite_contains(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let w = word.as_bytes();
    let mut i = 0;
    while let Some(pos) = haystack[i..].find(word) {
        let start = i + pos;
        let end = start + w.len();
        let before_ok = start == 0 || !is_ident(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_ident(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        i = start + 1;
    }
    false
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Canonicalise a path for comparison: axum's `:param` and the contract's
/// `{param}` both denote a single segment, and the contract uses camelCase
/// while axum routes are snake_case, so both are folded to a bare `{}`.
fn canon_path_string(p: &str) -> String {
    let mut out = String::with_capacity(p.len());
    let mut seg = String::new();
    for c in p.chars() {
        if c == '/' {
            out.push('/');
            if !seg.is_empty() {
                out.push_str(&fold(&seg));
                seg.clear();
            }
        } else {
            seg.push(c);
        }
    }
    if !seg.is_empty() {
        out.push_str(&fold(&seg));
    }
    out
}

fn fold(seg: &str) -> String {
    // Test for a parameter marker *before* stripping it: `:member_id` (axum)
    // and `{memberId}` (OpenAPI) both denote one segment and must collapse to
    // the same token.
    if seg.starts_with(':') || seg.starts_with('{') {
        return "{}".to_string();
    }
    // camelCase → lower, so a literal `memberId` and `member_id` compare equal.
    seg.to_ascii_lowercase().replace('_', "")
}

/// (path, methods) for every canonical path, from the OpenAPI document.
fn canonical_paths(root: &Path) -> BTreeMap<String, Vec<String>> {
    let src = fs::read_to_string(root.join("Contract/openapi/lcc-api-canonical.yaml"))
        .expect("read canonical openapi");
    let mut out = BTreeMap::new();
    let mut cur: Option<String> = None;
    for line in src.lines() {
        if let Some(p) = line.strip_prefix("  /") {
            if let Some(p) = p.strip_suffix(':') {
                // Fold on the way in so contract `{memberId}` and axum
                // `:member_id` land on the same key.
                cur = Some(canon_path_string(&format!("/api/v1/{p}")));
                out.entry(cur.clone().expect("just set"))
                    .or_insert_with(Vec::new);
            }
            continue;
        }
        if let Some(c) = cur.clone() {
            let t = line.trim_start();
            for verb in ["get", "post", "put", "patch", "delete"] {
                if t.starts_with(&format!("{verb}:")) {
                    out.entry(c.clone()).or_default().push(verb.to_uppercase());
                }
            }
        }
    }
    for v in out.values_mut() {
        v.sort();
        v.dedup();
    }
    out
}

/// (service, path) → methods, from every service router's `.route("…")`.
fn service_routes(root: &Path) -> BTreeMap<(String, String), Vec<String>> {
    let svc_root = root.join("Backend/engine/core/services");
    let mut out: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(&svc_root).expect("read services dir") {
        let entry = entry.expect("dir entry");
        if !entry.path().is_dir() {
            continue;
        }
        let svc = entry.file_name().to_string_lossy().to_string();
        for rel in ["src/http.rs", "src/http/mod.rs"] {
            let f = entry.path().join(rel);
            if !f.is_file() {
                continue;
            }
            let text = fs::read_to_string(&f).expect("read service http");
            collect_routes(&text, &svc, &mut out);
            break;
        }
    }
    out
}

fn collect_routes(text: &str, svc: &str, out: &mut BTreeMap<(String, String), Vec<String>>) {
    let mut pos = 0usize;
    while let Some(rel) = text[pos..].find(".route(") {
        let open = pos + rel + ".route(".len();
        let Some(q) = text[open..].find('"') else {
            break;
        };
        let path_at = open + q + 1;
        let Some(close) = text[path_at..].find('"') else {
            break;
        };
        let path = text[path_at..path_at + close].to_string();

        // The method expression runs to the `)` that closes `.route(`. We are
        // already past the opening paren, so depth starts at 0 and the first
        // unmatched `)` ends the expression.
        let mut depth = 0usize;
        let mut expr = String::new();
        let mut cur = path_at + close + 1;
        for c in text[cur..].chars() {
            match c {
                '(' => depth += 1,
                ')' if depth == 0 => break,
                ')' => depth -= 1,
                _ => {}
            }
            expr.push(c);
            cur += c.len_utf8();
        }

        let key = (svc.to_string(), path);
        out.entry(key).or_default().extend(methods_for(&expr));
        pos = cur;
    }
}

#[test]
fn every_service_route_is_declared_by_the_canonical_contract() {
    let root = repo_root();
    let canon = canonical_paths(&root);
    let routes = service_routes(&root);
    assert!(
        !canon.is_empty(),
        "canonical contract parsed to zero paths — parser regression"
    );

    let mut offenders: Vec<String> = Vec::new();
    for ((svc, path), methods) in &routes {
        if exempt(svc).is_some() || is_pending(svc) {
            continue;
        }
        if path.starts_with("/internal")
            || path.starts_with("/api/v1/ws/")
            || path.starts_with("/api/v1/sse/")
        {
            continue;
        }
        if matches!(
            path.as_str(),
            "/healthz" | "/readyz" | "/metrics" | "/health" | "/ready"
        ) {
            continue;
        }
        let c = canon_path(&canon, path);
        if c.is_none() {
            offenders.push(format!(
                "  {svc:<22} {:<58} -> NOT IN CONTRACT",
                format!("{} {}", methods.join(","), path)
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "service routes that the canonical contract does not declare:\n{}\n\n\
         These are unreachable through the gateway, which follows the contract \
         and forwards paths verbatim.",
        offenders.join("\n")
    );
}

/// Look a service path up in the contract, trying both the flat form and the
/// member-scoped form (`/api/v1/<domain>` vs `/api/v1/members/{id}/<domain>`).
fn canon_path(contract: &BTreeMap<String, Vec<String>>, path: &str) -> Option<Vec<String>> {
    let c = canon_path_string(path);
    if let Some(v) = contract.get(&c) {
        return Some(v.clone());
    }
    if let Some(rest) = c.strip_prefix("/api/v1/") {
        for prefix in ["members/{}/", "members/me/"] {
            let scoped = format!("/api/v1/{prefix}{rest}");
            if let Some(v) = contract.get(&scoped) {
                return Some(v.clone());
            }
        }
    }
    None
}

#[test]
fn every_migrated_service_route_matches_the_contract_method() {
    let root = repo_root();
    let canon = canonical_paths(&root);
    let routes = service_routes(&root);

    let mut offenders: Vec<String> = Vec::new();
    for ((svc, path), methods) in &routes {
        if exempt(svc).is_some()
            || is_pending(svc)
            || path.starts_with("/internal")
            || path.starts_with("/api/v1/ws/")
            || path.starts_with("/api/v1/sse/")
            || matches!(
                path.as_str(),
                "/healthz" | "/readyz" | "/metrics" | "/health" | "/ready"
            )
        {
            continue;
        }
        let Some(expected) = canon_path(&canon, path) else {
            continue; // reported by the test above
        };
        let mut actual: Vec<String> = methods
            .iter()
            .filter(|m| !is_omission(svc, path, m))
            .cloned()
            .collect();
        actual.sort();
        actual.dedup();
        if actual != expected {
            offenders.push(format!(
                "  {svc:<22} {:<58} -> service {actual:?} vs contract {expected:?}",
                path
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "service routes whose HTTP methods disagree with the contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn every_affected_service_uses_the_member_scoped_namespace() {
    // The core defect this file exists to prevent: a service sitting on the
    // flat `/api/v1/<domain>/…` shape that the gateway will never forward to.
    let root = repo_root();
    let routes = service_routes(&root);
    let mut flat: BTreeMap<String, usize> = BTreeMap::new();
    for (svc, path) in routes.keys() {
        if exempt(svc).is_some()
            || path.starts_with("/internal")
            || path.starts_with("/api/v1/ws/")
            || path.starts_with("/api/v1/sse/")
            || matches!(
                path.as_str(),
                "/healthz" | "/readyz" | "/metrics" | "/health" | "/ready"
            )
        {
            continue;
        }
        if !path.starts_with("/api/v1/members/") {
            *flat.entry(svc.clone()).or_default() += 1;
        }
    }

    let expected: BTreeMap<String, usize> = PENDING_MIGRATION
        .iter()
        .map(|(s, n)| ((*s).to_string(), *n))
        .collect();
    assert_eq!(
        flat, expected,
        "the flat-namespace migration ledger is out of date.\n\
         A service moved off the flat namespace -> remove it from \
         PENDING_MIGRATION and update its count.\n\
         A service gained a flat route -> that is the regression this test exists for."
    );
}

#[test]
fn the_pending_ledger_does_not_hide_anything_under_exempt() {
    // `exempt()` is the escape hatch, so it must not be able to quietly cover
    // a service that still needs migrating. Every exempt service must already
    // be canonical, or be infrastructure.
    let root = repo_root();
    let routes = service_routes(&root);
    let mut bad = Vec::new();
    for (svc, _) in PENDING_MIGRATION {
        assert!(
            exempt(svc).is_none(),
            "{svc} is in PENDING_MIGRATION but also exempt() — exempt it in one place only"
        );
    }
    for (svc, path) in routes.keys() {
        if PENDING_MIGRATION.iter().any(|(s, _)| s == svc) {
            continue;
        }
        if exempt(svc).is_some()
            && !path.starts_with("/api/v1/members/")
            && !path.starts_with("/api/v1/ws/")
            && !path.starts_with("/api/v1/sse/")
            && !path.starts_with("/api/v1/admin/")
            && !path.starts_with("/internal")
            && !path.starts_with("/api/v1/auth/")
            && !matches!(
                path.as_str(),
                "/healthz" | "/readyz" | "/metrics" | "/health" | "/ready"
            )
        {
            bad.push(format!("  {svc:<22} {path}"));
        }
    }
    assert!(
        bad.is_empty(),
        "exempt services must already be canonical:\n{}",
        bad.join("\n")
    );
}
#[test]
fn declared_surface_gaps_are_exactly_the_ledger() {
    // Recompute the omission set from the sources so CONTRACT_OMISSIONS cannot
    // go stale: an unlisted extra method fails here rather than being silently
    // tolerated.
    let root = repo_root();
    let canon = canonical_paths(&root);
    let routes = service_routes(&root);

    let mut found: BTreeSet<(String, String, String)> = BTreeSet::new();
    for ((svc, path), methods) in &routes {
        if exempt(svc).is_some()
            || is_pending(svc)
            || path.starts_with("/internal")
            || path.starts_with("/api/v1/ws/")
            || path.starts_with("/api/v1/sse/")
            || matches!(
                path.as_str(),
                "/healthz" | "/readyz" | "/metrics" | "/health" | "/ready"
            )
        {
            continue;
        }
        let Some(expected) = canon_path(&canon, path) else {
            continue;
        };
        for m in methods {
            if !expected.iter().any(|e| e == m) {
                found.insert((svc.clone(), path.clone(), m.clone()));
            }
        }
    }

    let declared: BTreeSet<(String, String, String)> = CONTRACT_OMISSIONS
        .iter()
        .flat_map(|(s, p, ms)| {
            ms.iter()
                .map(move |m| (s.to_string(), p.to_string(), m.to_string()))
        })
        .collect();

    assert_eq!(
        found, declared,
        "service surface that exceeds the contract changed.\n         Either the contract gains the operation, or the service stops serving it."
    );
}
