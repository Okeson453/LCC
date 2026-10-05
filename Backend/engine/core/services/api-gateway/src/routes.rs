//! The gateway's routing table, as data.
//!
//! The routes used to be hand-written directly into `http/router.rs`, which
//! made the routing surface impossible to verify against the canonical
//! contract: nothing but reading the builder chain told you what the gateway
//! actually served. That is how `GET /metrics` ended up declared in
//! `Contract/openapi/lcc-api-canonical.yaml` and recorded as live in
//! `Contract/docs/endpoint_contract_matrix.md` while never being registered —
//! the scrape endpoint 404'd.
//!
//! Every route is now declared once, here, and `http::router::build_router`
//! assembles the `axum::Router` from this table. Because the table is plain
//! data it can be checked against the contract mechanically, which
//! `tests/gateway_contract_conformance.rs` does: every canonical path must be
//! matched by a registered route.
//!
//! ## Ownership
//!
//! - [`RouteKind::Infra`] — served by the gateway itself (health/readiness/metrics).
//! - [`RouteKind::PublicAuth`] — the OAuth handshake entry points, which cannot
//!   require a Bearer token because the client has none yet.
//! - [`RouteKind::ProtectedProxy`] — forwarded to the owning upstream, behind
//!   `require_auth` + `rate_limit`.

/// HTTP methods a route accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    /// `axum::routing::any` — every method.
    Any,
}

impl Method {
    pub const fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Any => "*",
        }
    }
}

/// How a route is served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteKind {
    /// Handled in-process by the gateway; never proxied, never auth-gated.
    Infra,
    /// OAuth entry points. Unauthenticated by necessity.
    PublicAuth,
    /// Proxied to the owning upstream behind auth + rate limiting.
    ProtectedProxy,
}

#[derive(Debug, Clone, Copy)]
pub struct RouteSpec {
    /// `axum` path pattern (`/api/v1/content/*path`).
    pub pattern: &'static str,
    pub methods: &'static [Method],
    pub kind: RouteKind,
}

use Method::{Any, Get, Patch, Post};

/// The complete public routing surface of the api-gateway.
///
/// Kept in the same order as the canonical namespace so the two read together.
/// Domain coverage is by wildcard, which is deliberate: the contract nests most
/// of the surface under `/api/v1/members/{memberId}/<domain>/…`, and the
/// `UpstreamRegistry` (see `state::build_upstream_registry`) resolves which
/// service owns a given path.
pub const ROUTES: &[RouteSpec] = &[
    // ── INFRA ────────────────────────────────────────────────────────────
    // Declared by the canonical contract with `security: []`.
    RouteSpec {
        pattern: "/healthz",
        methods: &[Get],
        kind: RouteKind::Infra,
    },
    RouteSpec {
        pattern: "/readyz",
        methods: &[Get],
        kind: RouteKind::Infra,
    },
    RouteSpec {
        pattern: "/metrics",
        methods: &[Get],
        kind: RouteKind::Infra,
    },
    // ── AUTH (public) ────────────────────────────────────────────────────
    RouteSpec {
        pattern: "/api/v1/auth/linkedin/start",
        methods: &[Get],
        kind: RouteKind::PublicAuth,
    },
    RouteSpec {
        pattern: "/api/v1/auth/linkedin/callback",
        methods: &[Get],
        kind: RouteKind::PublicAuth,
    },
    RouteSpec {
        pattern: "/api/v1/auth/refresh",
        methods: &[Post],
        kind: RouteKind::PublicAuth,
    },
    RouteSpec {
        pattern: "/api/v1/auth/logout",
        methods: &[Post],
        kind: RouteKind::PublicAuth,
    },
    // ── MEMBERS ──────────────────────────────────────────────────────────
    // `/api/v1/members/me` must be matched before `/members/:member_id`, and
    // both before the `/members/:member_id/*path` wildcard, or `me` would be
    // captured as a member id.
    RouteSpec {
        pattern: "/api/v1/members/me",
        methods: &[Get, Patch],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/members/me/settings",
        methods: &[Get, Patch],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/members/:member_id",
        methods: &[Get],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/members/:member_id/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    // ── DOMAINS ──────────────────────────────────────────────────────────
    RouteSpec {
        pattern: "/api/v1/profile/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/content/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/engagement/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/contacts/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/opportunities/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/sequences/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/kb/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/analytics/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/briefing/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/approvals/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/audit/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/admin/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    // ── REALTIME ─────────────────────────────────────────────────────────
    RouteSpec {
        pattern: "/api/v1/ws/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    RouteSpec {
        pattern: "/api/v1/sse/*path",
        methods: &[Any],
        kind: RouteKind::ProtectedProxy,
    },
    // ── LEGACY (migration window) ────────────────────────────────────────
    // Internal governance eval, superseded by the per-resource
    // `/api/v1/approvals/{id}/decide` flow. Not in the canonical contract.
    RouteSpec {
        pattern: "/api/v1/admin/governor/evaluate",
        methods: &[Post],
        kind: RouteKind::ProtectedProxy,
    },
];

/// Routes served by the gateway itself, rather than proxied.
pub fn infra_routes() -> impl Iterator<Item = &'static RouteSpec> {
    ROUTES.iter().filter(|r| r.kind == RouteKind::Infra)
}

/// Routes forwarded to an upstream behind auth + rate limiting.
pub fn protected_routes() -> impl Iterator<Item = &'static RouteSpec> {
    ROUTES
        .iter()
        .filter(|r| r.kind == RouteKind::ProtectedProxy)
}

/// Public OAuth entry points.
pub fn public_auth_routes() -> impl Iterator<Item = &'static RouteSpec> {
    ROUTES.iter().filter(|r| r.kind == RouteKind::PublicAuth)
}

/// Translate an `axum` pattern into a matcher for a concrete canonical path.
///
/// `/api/v1/members/:member_id/content` → segment-wise, `:name` matches one
/// segment and `*name` matches the remainder. Used by the contract-conformance
/// test to prove every canonical path is reachable.
pub fn pattern_matches(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let seg: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let mut i = 0;
    while i < pat.len() {
        if pat[i].starts_with('*') {
            // Wildcard consumes one or more remaining segments.
            return i < seg.len();
        }
        if i >= seg.len() {
            return false;
        }
        if !pat[i].starts_with(':') && pat[i] != seg[i] {
            return false;
        }
        i += 1;
    }
    i == seg.len()
}
