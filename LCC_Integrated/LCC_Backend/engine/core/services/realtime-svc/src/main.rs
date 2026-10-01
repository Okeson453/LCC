//! lcc-realtime-svc entry point.

use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lcc_observability::init_tracing("lcc-realtime-svc");

    let cfg = lcc_realtime_svc::config::Config::from_env()?;
    let state = lcc_realtime_svc::db::AppState::new(cfg.clone()).await?;

    // Spawn the consumer that drains the Redis Stream and fans out to the
    // per-channel broadcast senders.
    let consumer_state = state.clone();
    tokio::spawn(async move {
        lcc_realtime_svc::events::run_consumer(consumer_state).await;
    });

    let app = lcc_realtime_svc::http::build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    info!(%addr, "lcc-realtime-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
