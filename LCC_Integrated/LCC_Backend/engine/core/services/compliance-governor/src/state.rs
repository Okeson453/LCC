//! AppState for the Compliance Governor service.

use crate::CandidateAction;
use crate::GovernorDeps;

impl GovernorDeps {
    /// Placeholder for tests; real construction in `main.rs`.
    ///
    /// Uses a deterministic Ed25519 keypair so the placeholder is reproducible
    /// and tests can construct both governor and verifier without needing a
    /// running Vault.
    pub fn placeholder() -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"lcc-compliance-governor-signing-v1:placeholder");
        let digest = hasher.finalize();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&digest[..32]);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);
        let signer =
            lcc_compliance::permit_token::signing_only_signer(signing_key, "integration-gateway");

        Self {
            config: std::sync::Arc::new(lcc_compliance::config::ComplianceConfig::default()),
            redis: make_placeholder_pool(),
            db: make_placeholder_pool_db(),
            permit_issuer: std::sync::Arc::new(signer),
            audit: lcc_audit_client::AuditClient::connect(
                lcc_audit_client::AuditClientConfig::default(),
            ),
            scoring_client: std::sync::Arc::new(crate::scoring::ScoringClient::placeholder()),
        }
    }
}

fn make_placeholder_pool() -> deadpool_redis::Pool {
    let cfg = deadpool_redis::Config::from_url("redis://localhost:6379");
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap_or_else(|_| {
            // If we can't create the pool, return one that errors on get()
            deadpool_redis::Config::from_url("redis://127.0.0.1:0")
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .expect("placeholder pool")
        })
}

fn make_placeholder_pool_db() -> sqlx::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_lazy("postgres://postgres:postgres@localhost:5432/lcc")
        .expect("placeholder pool")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_constructs() {
        let _d = GovernorDeps::placeholder();
    }
}
