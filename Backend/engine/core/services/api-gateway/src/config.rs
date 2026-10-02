//! api-gateway configuration.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiGatewayConfig {
    pub service_name: String,
    pub http_port: u16,
    pub environment: String,
    pub log_level: String,

    pub auth_jwt_audience: String,
    pub auth_jwt_issuer: String,
    pub auth_jwt_secret: String,

    pub rate_limit_per_minute: u32,

    pub audit_svc_url: String,
    pub orchestrator_url: String,
    pub identity_svc_url: String,
    pub profile_svc_url: String,
    pub content_svc_url: String,
    pub engagement_svc_url: String,
    pub network_svc_url: String,
    pub opportunity_svc_url: String,
    pub outreach_svc_url: String,
    pub analytics_svc_url: String,
    pub approval_svc_url: String,
    pub compliance_governor_url: String,
    pub kb_svc_url: String,
    pub realtime_svc_url: String,
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        Self {
            service_name: "api-gateway".into(),
            http_port: 8080,
            environment: "local".into(),
            log_level: "info".into(),
            auth_jwt_audience: "lcc-api".into(),
            auth_jwt_issuer: "lcc-identity-svc".into(),
            // F-AUDIT-10: the previous default was the literal
            // "dev-secret-change-me". Kept here ONLY so local dev still boots;
            // `from_env` below refuses to start on any non-local environment
            // while this value is in effect, so it can never reach staging or
            // production and mint tokens under a publicly-known HS256 key.
            auth_jwt_secret: DEV_ONLY_JWT_SECRET.into(),
            rate_limit_per_minute: 600,
            // F-AUDIT-37: every upstream URL below was `:8080`, but only
            // api-gateway itself binds 8080. The Core Engine services bind
            // 8081-8091 (see each service's `config.rs` `http_port` default,
            // and the matching `containerPort` in infra/k8s/base/*.yaml).
            // With all 10 pointed at 8080, every `/v1/<domain>/*` proxy route
            // was a connection-refused → 502.
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
}

/// The well-known development signing secret. Rejected outside `local`.
pub const DEV_ONLY_JWT_SECRET: &str = "dev-secret-change-me";

/// Environments in which the dev JWT secret is tolerated.
fn is_dev_profile(environment: &str) -> bool {
    matches!(environment, "local" | "dev" | "test")
}

impl ApiGatewayConfig {
    /// Build the config from the environment.
    ///
    /// Upstream service URLs and the JWT secret are read from the
    /// environment so a deployment does not silently inherit the localhost
    /// defaults. A missing/invalid JWT secret in a non-dev profile is a hard
    /// startup failure rather than a warning.
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();

        if let Ok(env) = std::env::var("LCC_ENVIRONMENT") {
            cfg.environment = env;
        }
        if let Ok(port) = std::env::var("LCC_HTTP_PORT") {
            cfg.http_port = port
                .parse()
                .map_err(|_| format!("LCC_HTTP_PORT is not a valid port: {port}"))?;
        }

        // Service URLs.
        let overrides: [(&str, fn(&mut Self) -> &mut String); 13] = [
            ("LCC_AUDIT_SVC_URL", |c| &mut c.audit_svc_url),
            ("LCC_IDENTITY_SVC_URL", |c| &mut c.identity_svc_url),
            ("LCC_PROFILE_SVC_URL", |c| &mut c.profile_svc_url),
            ("LCC_CONTENT_SVC_URL", |c| &mut c.content_svc_url),
            ("LCC_ENGAGEMENT_SVC_URL", |c| &mut c.engagement_svc_url),
            ("LCC_NETWORK_SVC_URL", |c| &mut c.network_svc_url),
            ("LCC_OPPORTUNITY_SVC_URL", |c| &mut c.opportunity_svc_url),
            ("LCC_OUTREACH_SVC_URL", |c| &mut c.outreach_svc_url),
            ("LCC_ANALYTICS_SVC_URL", |c| &mut c.analytics_svc_url),
            ("LCC_APPROVAL_SVC_URL", |c| &mut c.approval_svc_url),
            ("LCC_COMPLIANCE_GOVERNOR_URL", |c| &mut c.compliance_governor_url),
            ("LCC_KB_SVC_URL", |c| &mut c.kb_svc_url),
            ("LCC_ORCHESTRATOR_URL", |c| &mut c.orchestrator_url),
        ];
        for (var, field) in overrides {
            if let Ok(v) = std::env::var(var) {
                if !v.is_empty() {
                    *field(&mut cfg) = v;
                }
            }
        }

        // JWT secret — the one value that must never silently fall back.
        match std::env::var("LCC_AUTH_JWT_SECRET") {
            Ok(s) if !s.is_empty() => cfg.auth_jwt_secret = s,
            _ => {
                if !is_dev_profile(&cfg.environment) {
                    return Err(format!(
                        "LCC_AUTH_JWT_SECRET must be set in environment `{}`; \
                         the built-in dev secret is refused outside local/dev/test",
                        cfg.environment
                    ));
                }
                tracing::warn!(
                    "LCC_AUTH_JWT_SECRET not set; using the dev-only signing secret. \
                     This must never occur outside a local environment."
                );
            }
        }

        if let Ok(v) = std::env::var("LCC_AUTH_JWT_AUDIENCE") {
            if !v.is_empty() {
                cfg.auth_jwt_audience = v;
            }
        }
        if let Ok(v) = std::env::var("LCC_AUTH_JWT_ISSUER") {
            if !v.is_empty() {
                cfg.auth_jwt_issuer = v;
            }
        }
        if let Ok(v) = std::env::var("LCC_RATE_LIMIT_PER_MINUTE") {
            cfg.rate_limit_per_minute = v
                .parse()
                .map_err(|_| "LCC_RATE_LIMIT_PER_MINUTE is not a valid integer".to_string())?;
        }

        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_secret_is_flagged_as_dev_only() {
        // Guards against the sentinel being renamed without updating the check.
        assert_eq!(ApiGatewayConfig::default().auth_jwt_secret, DEV_ONLY_JWT_SECRET);
    }

    #[test]
    fn non_dev_profiles_are_recognised() {
        assert!(is_dev_profile("local"));
        assert!(is_dev_profile("dev"));
        assert!(is_dev_profile("test"));
        assert!(!is_dev_profile("staging"));
        assert!(!is_dev_profile("production"));
        assert!(!is_dev_profile("prod"));
    }
}

/// Alias so tooling and tests can refer to every service's config by the
/// same name.
pub type Config = ApiGatewayConfig;
