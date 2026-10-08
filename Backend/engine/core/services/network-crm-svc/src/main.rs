use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tracing is best-effort: a malformed RUST_LOG must not stop the
    // service from serving, so a failure is reported and startup continues.
    if let Err(e) = lcc_observability::init_tracing("lcc-network-crm-svc") {
        eprintln!("[lcc-network-crm-svc] tracing init failed: {e}");
    }
    let cfg = lcc_network_crm_svc::config::Config::from_env()?;
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await?;
    let redis = deadpool_redis::Config::from_url(&cfg.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| format!("redis pool: {e}"))?;
    // The verifier is built once at startup and handed to the router, rather
    // than each handler reading LCC_AUTH_JWT_SECRET per request: the decoding
    // key is then parsed a single time, and the secret becomes an explicit
    // input that tests can substitute instead of process-global state.
    let secret = std::env::var("LCC_AUTH_JWT_SECRET")
        .map_err(|_| "LCC_AUTH_JWT_SECRET not set".to_string())?;
    let verifier = std::sync::Arc::new(lcc_auth::JwtVerifier::new(
        secret.as_bytes(),
        lcc_auth::DEFAULT_JWT_ISSUER,
        lcc_auth::DEFAULT_JWT_AUDIENCE,
    ));
    let state = lcc_network_crm_svc::state::AppState::new(db, redis).await;
    let app = lcc_network_crm_svc::http::build_router(state, verifier);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    tracing::info!(%addr, "lcc-network-crm-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
