//! WebSocket handlers.
//!
//! Per-channel `/api/v1/ws/<channel>` upgrades:
//!   - Authenticate via `Authorization: Bearer <jwt>` (same secret as
//!     api-gateway and identity-svc).
//!   - Subscribe the verified member_id to the per-channel broadcast.
//!   - Stream `EventEnvelope`s to the client as JSON text frames.
//!   - Honour the heartbeat cadence (server pings every 30s; client may
//!     reply with `{"type":"ping"}` and we send `{"type":"pong"}`).
//!   - Honour the idle timeout (close with 1001 after no traffic).
//!   - Enforce per-member connection limit; over-limit returns 1013.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::auth::{self, JwtClaims};
use crate::domain::Channel;
use crate::error::Error;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Optional reconnect cursor — last event_id the client has seen. Used
    /// to resume after a reconnect. We don't replay events here (the
    /// consumer dedup state already handles late arrivals), but the client
    /// can use this to know which events to keep.
    pub last_event_id: Option<String>,
    /// Optional explicit channel override (otherwise derived from URL path).
    pub channel: Option<String>,
}

/// GET /api/v1/ws/{channel}
pub async fn ws_handler(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Query(q): Query<WsQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, Error> {
    let token = auth::extract_bearer(headers.get(axum::http::header::AUTHORIZATION))?;
    let claims = auth::verify(
        &token,
        &state.config().auth_jwt_secret,
        &state.config().jwt_issuer,
        &state.config().jwt_audience,
    )?;
    let member_id = claims.sub_uuid()?;

    let ch = Channel::from_id(&channel)
        .or_else(|| q.channel.as_deref().and_then(Channel::from_id))
        .ok_or_else(|| Error::NotFound(format!("unknown channel: {channel}")))?;

    // Enforce per-member connection limit.
    {
        let mut conns = state.member_conns().await;
        let count = conns.entry(member_id).or_insert(0);
        if *count >= state.config().per_member_conn_limit {
            return Err(Error::Forbidden(format!(
                "per_member_conn_limit reached ({})",
                state.config().per_member_conn_limit
            )));
        }
        *count += 1;
    }

    let response = ws
        .on_upgrade(move |socket| async move {
            if let Err(e) = run_socket(socket, state.clone(), member_id, claims, ch).await {
                warn!(error = %e, member_id = %member_id, "ws connection ended with error");
            }
            // Decrement the per-member counter.
            let mut conns = state.member_conns().await;
            if let Some(c) = conns.get_mut(&member_id) {
                *c = c.saturating_sub(1);
                if *c == 0 {
                    conns.remove(&member_id);
                }
            }
        })
        .into_response();
    Ok(response)
}

async fn run_socket(
    socket: WebSocket,
    state: AppState,
    member_id: uuid::Uuid,
    claims: JwtClaims,
    channel: Channel,
) -> Result<(), Error> {
    info!(%member_id, channel = channel.id(), "ws connection established");

    let mut rx = state.broadcast(channel)?.subscribe();

    // Split the socket.
    let (mut sink, mut stream) = socket.split();

    // Build an initial hello frame so the client knows which channel it's
    // subscribed to and what member_id is associated with the connection.
    let hello = serde_json::json!({
        "type": "hello",
        "channel": channel.id(),
        "member_id": member_id.to_string(),
        "role": claims.role,
        "heartbeat_seconds": state.config().heartbeat_seconds,
        "idle_timeout_seconds": state.config().idle_timeout_seconds,
    });
    sink.send(Message::Text(hello.to_string()))
        .await
        .map_err(|e| Error::Internal(format!("ws send: {e}")))?;

    let heartbeat_interval = Duration::from_secs(state.config().heartbeat_seconds);
    let idle_timeout = Duration::from_secs(state.config().idle_timeout_seconds);
    let mut last_activity = Instant::now();

    loop {
        tokio::select! {
            // Server → client: broadcast events or heartbeats.
            biased;

            event = rx.recv() => {
                match event {
                    Ok(env) => {
                        if env.member_id != member_id {
                            // Per-member scoping: a broadcast is shared
                            // across all subscribers to the channel, but
                            // each client only receives events for its own
                            // member_id. The Consumer (broadcast sender)
                            // fans out everything; the connection layer
                            // enforces per-member filtering.
                            continue;
                        }
                        let frame = serde_json::to_string(&env).unwrap_or_default();
                        sink.send(Message::Text(frame))
                            .await
                            .map_err(|e| Error::Internal(format!("ws send: {e}")))?;
                        last_activity = Instant::now();
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Skip — receiver lagged; events are best-effort.
                        debug!("ws lagged; continuing");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        return Err(Error::Internal("broadcast closed".into()));
                    }
                }
            }

            // Client → server: handle client frames (ping/pong, subscribe/unsubscribe).
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_activity = Instant::now();
                        if let Ok(frame) = serde_json::from_str::<ClientFrame>(&text) {
                            match frame.frame_type.as_str() {
                                "ping" => {
                                    let pong = serde_json::json!({"type":"pong"});
                                    sink.send(Message::Text(pong.to_string())).await.ok();
                                }
                                "pong" => {
                                    // Client replied to our ping.
                                }
                                _ => {
                                    // Unknown frame types are ignored (not an
                                    // error — they may be from a future
                                    // protocol version).
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        return Ok(());
                    }
                    Some(Ok(Message::Ping(_))) => {
                        // axum handles ping/pong at the WS level.
                        last_activity = Instant::now();
                    }
                    Some(Ok(_)) => {
                        // Other frame types (binary, pong) — accept and continue.
                        last_activity = Instant::now();
                    }
                    Some(Err(e)) => {
                        return Err(Error::Internal(format!("ws recv: {e}")));
                    }
                    None => {
                        return Ok(());
                    }
                }
            }

            // Heartbeat tick — send server ping if no traffic for the
            // heartbeat interval.
            _ = tokio::time::sleep(heartbeat_interval) => {
                if last_activity.elapsed() >= heartbeat_interval {
                    let ping = serde_json::json!({"type":"ping","ts": chrono::Utc::now().to_rfc3339()});
                    if let Err(e) = sink.send(Message::Text(ping.to_string())).await {
                        return Err(Error::Internal(format!("ws ping send: {e}")));
                    }
                }
                // Idle close.
                if last_activity.elapsed() >= idle_timeout {
                    info!(%member_id, "ws idle timeout; closing");
                    let _ = sink.send(Message::Close(None)).await;
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientFrame {
    #[serde(rename = "type")]
    pub frame_type: String,
}
