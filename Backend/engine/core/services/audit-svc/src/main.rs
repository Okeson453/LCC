use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lcc_observability::init_tracing("lcc-audit-svc");
    let cfg = lcc_audit_svc::config::Config::from_env()?;
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&cfg.database_url).await?;
    let state = lcc_audit_svc::state::AppState::new(db).await;
    let app = lcc_audit_svc::http::build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    tracing::info!(%addr, "lcc-audit-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
