//! Integration Gateway service entrypoint.

use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use lcc_observability::tracing_init::{init_tracing, TracingConfig};
use lcc_security::vault::VaultClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod audit;
mod backoff;
mod circuit_breaker;
mod config;
mod error;
mod health;
mod idempotency;
mod permit;
mod restriction;
mod router;
mod state;
mod track_a;
mod track_b;

use crate::config::IntegrationGatewayConfig;

/// Correlation map for in-flight Track B browser-extension actions: an action
/// id to the channel waiting for the human confirmation. Named so the router,
/// the HTTP handler and the WS task all agree on one type.
type PendingConfirmations = Arc<
    tokio::sync::Mutex<
        std::collections::HashMap<
            uuid::Uuid,
            tokio::sync::oneshot::Sender<track_b_executor::ConfirmationOutcome>,
        >,
    >,
>;
use crate::permit::verifier::PermitVerifier;
use crate::state::IntegrationGatewayState;
use crate::track_a as track_a_executor;
use crate::track_b as track_b_executor;

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteActionRequestDto {
    permit_token: String,
    action_id: uuid::Uuid,
    member_id: uuid::Uuid,
    action_type: String,
    idempotency_key: String,
    is_organization: bool,
    payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackBRequestDto {
    permit_token: String,
    action_id: uuid::Uuid,
    member_id: uuid::Uuid,
    action_type: String,
    idempotency_key: String,
    target_contact_id: Option<uuid::Uuid>,
    target_post_id: Option<String>,
    fields: serde_json::Value,
    timeout_seconds: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Tracing.
    init_tracing(TracingConfig::default())?;

    // 2. Service config.
    let config = IntegrationGatewayConfig::from_env();
    tracing::info!(http_port = config.http_port, "integration-gateway starting");

    // 3. Build pools.
    let db = lcc_db::build_pool(&lcc_db::PoolConfig {
        url: config.database_url.clone(),
        ..lcc_db::PoolConfig::default()
    })
    .await?;
    let redis_pool = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| format!("redis pool: {e}"))?;

    // 4. Vault client (only this service reads LinkedIn token paths).
    let vault = Arc::new(VaultClient::new(&config.vault_addr, &config.vault_token));

    // 5. Permit verifier — assembled from the public half of the Governor's
    //    Ed25519 keypair. Refuses to start without a configured public key.
    let permit_verifier = Arc::new(
        PermitVerifier::from_public_key_bytes(&config.permit_pubkey_b64)
            .map_err(|e| format!("permit verifier: {e}"))?,
    );

    // 6. Idempotency store.
    let idempotency = Arc::new(crate::idempotency::IdempotencyStore::new(
        redis_pool.clone(),
        db.clone(),
    ));

    // 7. Rate limiter.
    let rate_limiter =
        Arc::new(lcc_integrations::limiter::RateLimiter::with_default_linkedin_limits());

    // 8. Compliance config.
    let compliance_config = std::sync::Arc::new(
        lcc_compliance::config::ComplianceConfig::from_yaml_file(&config.compliance_config_path)?,
    );

    // 9. Audit client.
    let audit =
        lcc_audit_client::AuditClient::connect(lcc_audit_client::AuditClientConfig::default());

    // 10. Pending confirmation map for Track B.
    let pending = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    // 11. State.
    let state = Arc::new(IntegrationGatewayState {
        db: db.clone(),
        redis: redis_pool.clone(),
        config: compliance_config,
        vault,
        permit_verifier,
        idempotency,
        rate_limiter,
        audit,
    });

    // 12. Router.
    // State is attached last, so the router must already be keyed on
    // `Arc<IntegrationGatewayState>` while the routes are being added.
    let app = Router::<Arc<IntegrationGatewayState>>::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(readyz))
        .route("/internal/integration/execute", post(execute_track_a_http))
        .route(
            "/internal/integration/execute-track-b",
            post(execute_track_b_http),
        )
        .layer(Extension(pending.clone()))
        .with_state(state.clone());

    // 13. Track B WSS listener (separate from HTTP).
    let track_b_pending = pending.clone();
    let track_b_state = state.clone();
    let track_b_addr: std::net::SocketAddr = config.track_b_ws_addr.parse()?;
    tokio::spawn(async move {
        if let Err(e) = run_track_b_ws(track_b_addr, track_b_state, track_b_pending).await {
            tracing::error!(error = %e, "track_b ws server crashed");
        }
    });

    // 14. Serve HTTP.
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.http_port)).await?;
    tracing::info!(addr = ?listener.local_addr()?, "integration-gateway serving");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Map an executor error onto an HTTP status.
///
/// A locally-decided budget rejection is a 429 so the caller can back off; a
/// breaker that is open is a 503, because the service is temporarily unable
/// rather than the caller asking too much. Everything else stays a 500.
fn status_for(e: &error::IntegrationError) -> axum::http::StatusCode {
    use crate::error::IntegrationError;
    use axum::http::StatusCode;
    match e {
        IntegrationError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
        IntegrationError::CircuitOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn readyz(
    State(state): State<Arc<IntegrationGatewayState>>,
) -> impl axum::response::IntoResponse {
    health::readyz(State(state)).await
}

async fn execute_track_a_http(
    State(state): State<Arc<IntegrationGatewayState>>,
    Json(req): Json<ExecuteActionRequestDto>,
) -> Result<Json<track_a_executor::ExecuteActionResponse>, axum::http::StatusCode> {
    let req = track_a_executor::ExecuteActionRequest {
        permit_token: req.permit_token,
        action_id: req.action_id,
        member_id: req.member_id,
        action_type: req.action_type,
        idempotency_key: req.idempotency_key,
        is_organization: req.is_organization,
        payload: req.payload,
    };
    track_a_executor::execute(state, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!(error = %e, "execute_track_a failed");
            status_for(&e)
        })
}

async fn execute_track_b_http(
    State(state): State<Arc<IntegrationGatewayState>>,
    // The pending-confirmation map rides as an `Extension`, not as router
    // state: the rest of the router is keyed on `Arc<IntegrationGatewayState>`,
    // and axum allows only one state type per router.
    Extension(pending): Extension<PendingConfirmations>,
    Json(req): Json<TrackBRequestDto>,
) -> Result<Json<track_b_executor::ExecuteActionResponse>, axum::http::StatusCode> {
    let req = track_b_executor::ExecuteActionRequest {
        permit_token: req.permit_token,
        action_id: req.action_id,
        member_id: req.member_id,
        action_type: req.action_type,
        idempotency_key: req.idempotency_key,
        target_contact_id: req.target_contact_id,
        target_post_id: req.target_post_id,
        fields: req.fields,
        timeout_seconds: req.timeout_seconds,
    };
    track_b_executor::execute(state, pending, req)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!(error = %e, "execute_track_b failed");
            status_for(&e)
        })
}

async fn run_track_b_ws(
    addr: std::net::SocketAddr,
    _state: Arc<IntegrationGatewayState>,
    _pending: PendingConfirmations,
) -> Result<(), Box<dyn std::error::Error>> {
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "track_b WSS server listening");

    while let Ok((stream, peer)) = listener.accept().await {
        tracing::info!(peer = %peer, "track_b client connected");
        // Clone per connection: `tokio::spawn` takes ownership, and the loop
        // must keep its own handle for the next accepted socket.
        let pending = _pending.clone();
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(tokio_tungstenite::tungstenite::Message::Text(s)) => {
                                if let Err(e) =
                                    track_b_executor::handle_extension_frame(pending.clone(), &s)
                                        .await
                                {
                                    tracing::warn!(error = %e, "track_b handle failed");
                                }
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Ping(p)) => {
                                let _ = write
                                    .send(tokio_tungstenite::tungstenite::Message::Pong(p))
                                    .await;
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                            _ => {}
                        }
                    }
                }
                Err(e) => tracing::warn!(error = %e, "track_b ws upgrade failed"),
            }
        });
    }
    Ok(())
}
