use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tracing is best-effort: a malformed RUST_LOG must not stop the
    // service from serving, so a failure is reported and startup continues.
    if let Err(e) = lcc_observability::init_tracing("lcc-analytics-svc") {
        eprintln!("[lcc-analytics-svc] tracing init failed: {e}");
    }
    let cfg = lcc_analytics_svc::config::Config::from_env()?;
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await?;
    let state = lcc_analytics_svc::state::AppState::new(db).await;
    let app = lcc_analytics_svc::http::build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    tracing::info!(%addr, "lcc-analytics-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
