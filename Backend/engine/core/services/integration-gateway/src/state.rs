//! AppState for the Integration Gateway.

use deadpool_redis::Pool;
use lcc_compliance::config::ComplianceConfig;
use lcc_integrations::limiter::RateLimiter;
use lcc_security::vault::VaultClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct IntegrationGatewayState {
    pub db: sqlx::PgPool,
    pub redis: Pool,
    pub config: Arc<ComplianceConfig>,
    pub vault: Arc<VaultClient>,
    pub permit_verifier: Arc<super::permit::verifier::PermitVerifier>,
    pub idempotency: Arc<super::idempotency::IdempotencyStore>,
    pub rate_limiter: Arc<RateLimiter>,
    pub audit: lcc_audit_client::AuditClient,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_compiles() {
        // Smoke test: type signature compiles.
        fn _accept(_s: IntegrationGatewayState) {}
    }
}
