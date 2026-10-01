use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tracing is best-effort: a malformed RUST_LOG must not stop the
    // service from serving, so a failure is reported and startup continues.
    if let Err(e) = lcc_observability::init_tracing("lcc-kb-svc") {
        eprintln!("[lcc-kb-svc] tracing init failed: {e}");
    }
    let cfg = lcc_kb_svc::config::Config::from_env()?;
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await?;
    let redis = deadpool_redis::Config::from_url(&cfg.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| format!("redis pool: {e}"))?;
    let state = lcc_kb_svc::state::AppState::new(db, redis).await;
    let app = lcc_kb_svc::http::build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    tracing::info!(%addr, "lcc-kb-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
