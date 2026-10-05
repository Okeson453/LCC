//! Shared application state.

use std::sync::Arc;

use crate::config::ApiGatewayConfig;
use crate::middleware::rate_limit::RateLimiter;
use crate::proxy::{pool::build_pools, pool::UpstreamConfig, RouteBinding, UpstreamRegistry};

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

struct Inner {
    pub config: ApiGatewayConfig,
    pub http_client: reqwest::Client,
    /// F-AUDIT-01: domain-prefix → upstream bindings, built once at startup.
    /// Previously `proxy/pool.rs` was never constructed, so every protected
    /// route short-circuited to a fake `{"status":"ok"}`.
    pub upstreams: UpstreamRegistry,
    /// F-AUDIT-05: shared per-process rate limiter. Previously a fresh
    /// (always-empty) limiter was constructed per request, so the limit could
    /// never be reached.
    pub rate_limiter: RateLimiter,
    /// F-AUDIT-07: the JWT verifier. The previous `require_auth` called
    /// `state.jwt_secret()` / `state.jwt_audience()`, neither of which existed.
    pub jwt_verifier: std::sync::Arc<lcc_auth::JwtVerifier>,
}

/// Per-upstream tuning. `timeout_ms` is deliberately short: the gateway is on
/// the request hot path (Technical Design Spec §26 requires predictable
/// low-latency behaviour under load), and a slow downstream should surface as
/// a 504 the client can retry rather than a hung request.
pub const UPSTREAM_TIMEOUT_MS: u64 = 15_000;
pub const UPSTREAM_MAX_CONNECTIONS: usize = 64;

/// Build the registry from the service URLs declared in config.
///
/// Per the canonical namespace `/api/v1/<domain>/...`, the gateway forwards
/// each request to the upstream service that owns that domain. Each upstream
/// implements the canonical paths internally (e.g., identity-svc listens at
/// `/api/v1/auth/...` and `/api/v1/members/...`).
pub fn build_upstream_registry(config: &ApiGatewayConfig) -> UpstreamRegistry {
    let declarations: Vec<(&'static str, UpstreamConfig)> = vec![
        // Auth + Members both live on identity-svc
        (
            "auth",
            UpstreamConfig::new("identity-svc", &config.identity_svc_url),
        ),
        (
            "members",
            UpstreamConfig::new("identity-svc", &config.identity_svc_url),
        ),
        (
            "profile",
            UpstreamConfig::new("profile-svc", &config.profile_svc_url),
        ),
        (
            "content",
            UpstreamConfig::new("content-svc", &config.content_svc_url),
        ),
        (
            "engagement",
            UpstreamConfig::new("engagement-svc", &config.engagement_svc_url),
        ),
        // Network/CRM exposes /contacts
        (
            "contacts",
            UpstreamConfig::new("network-crm-svc", &config.network_svc_url),
        ),
        // Opportunity Svc exposes /opportunities
        (
            "opportunities",
            UpstreamConfig::new("opportunity-svc", &config.opportunity_svc_url),
        ),
        // Outreach Svc exposes /sequences
        (
            "sequences",
            UpstreamConfig::new("outreach-svc", &config.outreach_svc_url),
        ),
        (
            "analytics",
            UpstreamConfig::new("analytics-svc", &config.analytics_svc_url),
        ),
        (
            "briefing",
            UpstreamConfig::new("orchestrator", &config.orchestrator_url),
        ),
        (
            "approvals",
            UpstreamConfig::new("approval-svc", &config.approval_svc_url),
        ),
        (
            "audit",
            UpstreamConfig::new("audit-svc", &config.audit_svc_url),
        ),
        // KB records live on profile-svc OR a dedicated kb-svc (configurable);
        // here we forward to kb-svc.
        ("kb", UpstreamConfig::new("kb-svc", &config.kb_svc_url)),
        // Admin (compliance config + restriction state) lives on the
        // compliance-governor.
        (
            "admin",
            UpstreamConfig::new("compliance-governor", &config.compliance_governor_url),
        ),
        // Realtime: WS upgrades go to a dedicated realtime-svc that runs the
        // 5 dashboard channels. SSE fallback also served from the same host.
        (
            "realtime",
            UpstreamConfig::new("realtime-svc", &config.realtime_svc_url),
        ),
    ];

    let bindings = declarations
        .into_iter()
        .map(|(prefix, cfg)| RouteBinding {
            prefix,
            pool: build_pools(vec![cfg]).remove(0),
        })
        .collect();

    UpstreamRegistry::new(bindings)
}

impl AppState {
    pub fn new(config: ApiGatewayConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(UPSTREAM_MAX_CONNECTIONS)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let upstreams = build_upstream_registry(&config);
        let rate_limiter = RateLimiter::new(60_000, config.rate_limit_per_minute as u64);

        // F-AUDIT-10: `auth_jwt_secret` defaults to the literal string
        // "dev-secret-change-me". A deployment that forgets to override it
        // would mint and accept tokens under a publicly-known HS256 key.
        // `ApiGatewayConfig::from_env` now refuses to start on that default
        // outside a local/dev profile; see config.rs.
        let jwt_verifier = std::sync::Arc::new(lcc_auth::JwtVerifier::new(
            config.auth_jwt_secret.as_bytes(),
            config.auth_jwt_issuer.clone(),
            config.auth_jwt_audience.clone(),
        ));

        Self(Arc::new(Inner {
            config,
            http_client,
            upstreams,
            rate_limiter,
            jwt_verifier,
        }))
    }

    pub fn config(&self) -> &ApiGatewayConfig {
        &self.0.config
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.0.http_client
    }

    pub fn upstreams(&self) -> &UpstreamRegistry {
        &self.0.upstreams
    }

    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.0.rate_limiter
    }

    pub fn jwt_verifier(&self) -> &std::sync::Arc<lcc_auth::JwtVerifier> {
        &self.0.jwt_verifier
    }

    pub fn jwt_audience(&self) -> &str {
        &self.0.config.auth_jwt_audience
    }
}
