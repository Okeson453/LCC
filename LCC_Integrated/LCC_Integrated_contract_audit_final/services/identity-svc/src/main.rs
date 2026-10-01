//! identity-svc entry point.

use std::net::SocketAddr;

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = match lcc_identity_svc::config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e}");
            // Fall back to a local-dev config if we explicitly opt in.
            if std::env::var("LCC_ENVIRONMENT").as_deref() == Ok("local")
                || std::env::var("LCC_ENVIRONMENT").as_deref() == Ok("test")
            {
                lcc_identity_svc::config::Config::dev_local()?
            } else {
                return Err(e.into());
            }
        }
    };

    lcc_observability::init_tracing("lcc-identity-svc");

    let state = lcc_identity_svc::state::AppState::new(cfg.clone()).await?;
    let app = lcc_identity_svc::http::router::build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    info!(%addr, "lcc-identity-svc listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
