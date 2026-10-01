//! Compliance Governor service entrypoint.

use lcc_compliance::permit_token::{generate_keypair, signing_only_signer, PermitSigner};
use lcc_observability::metrics::Metrics;
use lcc_observability::tracing_init::{init_tracing, TracingConfig};
use std::sync::Arc;

// The service body lives in the library target (`src/lib.rs`). Declaring the
// modules again here compiled a second, divergent copy of the crate whose root
// had none of the items `lib.rs` defines (`GovernorDeps`, `AccountState`, …)
// and which had no `http` module at all.
use compliance_governor::{
    config::GovernorServiceConfig,
    error::GovernorError,
    health::started_at,
    http,
    scoring::ScoringClient,
    GovernorDeps,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Tracing.
    if let Err(e) = init_tracing(TracingConfig::default()) {
        // Best-effort: a malformed RUST_LOG must not stop the governor.
        eprintln!("[compliance-governor] tracing init failed: {e}");
    }

    // 2. Service config.
    let service_config = GovernorServiceConfig::from_env();
    tracing::info!(
        http_port = service_config.http_port,
        grpc_port = service_config.grpc_port,
        "compliance-governor starting"
    );

    // 3. Load active compliance config (load-time, not runtime-negotiable — axiom 4).
    let compliance_config = service_config.load_compliance_config()?;
    compliance_config.validate()?;
    tracing::info!(
        version = %compliance_config.version,
        "loaded active compliance config"
    );

    // 4. Build pools.
    let db = lcc_db::build_pool(&lcc_db::PoolConfig {
        url: service_config.database_url.clone(),
        ..lcc_db::PoolConfig::default()
    })
    .await?;
    let redis_cfg = deadpool_redis::Config::from_url(&service_config.redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| GovernorError::Config(format!("redis pool: {e}")))?;

    // 5. Permit token issuer. In production this key comes from the
    //    `permit-token-signing-key` ExternalSecret (Vault transit); for local
    //    dev we derive it from the configured seed so the Gateway and Governor
    //    can be regenerated together. The asymmetric Ed25519 split means the
    //    Gateway can verify without ever being able to sign (see ADR-0003).
    let permit_issuer: Arc<PermitSigner> = if let Some(seed) = service_config.permit_signing_seed.as_ref() {
        // Deterministic dev/test path: derive a key from a 32-byte seed.
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"lcc-compliance-governor-signing-v1:");
        hasher.update(seed.as_bytes());
        let digest = hasher.finalize();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&digest[..32]);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);
        Arc::new(signing_only_signer(signing_key, "integration-gateway"))
    } else {
        // Production path: pull the signing key from Vault via the ExternalSecret.
        // We refuse to start without a seed in dev mode either — see F-14.
        if let Ok(secret_b64) = std::env::var("PERMIT_SIGNING_KEY_B64") {
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            let bytes = STANDARD.decode(secret_b64.trim()).map_err(|e| {
                GovernorError::Config(format!("permit signing key decode: {e}"))
            })?;
            let mut key_bytes = [0u8; 32];
            if bytes.len() != 32 {
                return Err(Box::new(GovernorError::Config(
                    "PERMIT_SIGNING_KEY_B64 must decode to 32 bytes".into(),
                )) as Box<dyn std::error::Error>);
            }
            key_bytes.copy_from_slice(&bytes);
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);
            Arc::new(signing_only_signer(signing_key, "integration-gateway"))
        } else {
            // No key configured — fail closed (axiom 4 / F-14).
            return Err(Box::new(GovernorError::Config(
                "PERMIT_SIGNING_KEY_B64 (or PERMIT_SIGNING_SEED for dev) is required".into(),
            )) as Box<dyn std::error::Error>);
        }
    };

    // 6. Audit client.
    let audit = lcc_audit_client::AuditClient::connect(lcc_audit_client::AuditClientConfig::default());

    // 7. Scoring client.
    let scoring_client = Arc::new(ScoringClient::new(service_config.scoring_intel_endpoint.clone()));

    // 8. Assemble dependencies.
    let deps = Arc::new(GovernorDeps {
        config: Arc::new(compliance_config),
        redis: redis_pool,
        db: db.clone(),
        permit_issuer,
        audit,
        scoring_client,
    });

    // 9. Metrics.
    let metrics = Arc::new(Metrics::new("compliance-governor")?);

    // 10. Router — canonical REST namespace per lcc-api-canonical.yaml.
    let app = http::build_router(deps.clone(), metrics.clone());

    // 11. Serve.
    let _ = started_at(); // initialize uptime tracking
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", service_config.http_port)).await?;
    tracing::info!(addr = ?listener.local_addr()?, "compliance-governor serving");
    axum::serve(listener, app).await?;

    Ok(())
}
