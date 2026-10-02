#[derive(Debug, Clone)]
pub struct Config {
    /// Logical service name, used for tracing, metrics and the
    /// `x-service` header. Fixed per service, not per deployment.
    pub service_name: String,
    /// Where this service sends audit events.
    pub audit_svc_url: String,
    /// The Compliance Governor, which every external action is gated on.
    pub compliance_governor_url: String,

    pub database_url: String,
    pub redis_url: String,
    pub http_port: u16,
}
/// Documented local-development defaults. The ports match the `containerPort`
/// pinned in `infra/k8s/base/*.yaml`, and the upstream URLs match the
/// api-gateway's `Default`, so a local stack and a k8s stack agree.
impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: "engagement-svc".into(),
            audit_svc_url: "http://audit-svc:8091".into(),
            compliance_governor_url: "http://compliance-governor:8080".into(),
            database_url: "postgres://lcc:lcc@postgres:5432/lcc".into(),
            redis_url: "redis://redis:6379".into(),
            http_port: 8084,
        }
    }
}


impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            service_name: "engagement-svc".into(),
            audit_svc_url: std::env::var("LCC_AUDIT_SVC_URL")
                .unwrap_or_else(|_| "http://audit-svc:8091".to_string()),
            compliance_governor_url: std::env::var("LCC_COMPLIANCE_GOVERNOR_URL")
                .unwrap_or_else(|_| "http://compliance-governor:8080".to_string()),
            database_url: std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set".to_string())?,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            http_port: std::env::var("LCC_HTTP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8084),
        })
    }
}

