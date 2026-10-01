//! api-gateway main.

use std::net::SocketAddr;

use lcc_api_gateway::{config::ApiGatewayConfig, http::router, state::AppState, telemetry};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // F-AUDIT-10: `from_env` returns a Result and refuses to start with the
    // built-in dev JWT secret outside a local/dev/test profile, so a
    // misconfigured deployment fails fast instead of minting tokens under a
    // publicly-known HS256 key.
    let config = ApiGatewayConfig::from_env()?;
    telemetry::init(&config.service_name, &config.log_level);

    let state = AppState::new(config.clone());
    info!(
        domains = ?state.upstreams().bound_prefixes(),
        "upstream bindings resolved"
    );

    // `ConnectInfo` is required so the rate limiter can fall back to the peer
    // address for traffic that is not yet authenticated.
    let app = router::build_router(state).into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.http_port));
    info!(%addr, service = %config.service_name, "listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
