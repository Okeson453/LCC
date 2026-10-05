//! SSE fallback handlers.
//!
//! Per contract: when WebSocket is not feasible, the client should be able to
//! fall back to Server-Sent Events. We serve `/api/v1/sse/<channel>` with the
//! same auth model (Bearer JWT) and the same per-member scoping.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use serde::Serialize;
use std::convert::Infallible;
use std::time::Duration;
use tracing::info;

use crate::auth;
use crate::domain::Channel;
use crate::error::Error;
use crate::state::AppState;

pub async fn sse_handler(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    headers: HeaderMap,
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
        .ok_or_else(|| Error::NotFound(format!("unknown channel: {channel}")))?;

    info!(%member_id, channel = ch.id(), "sse connection established");

    let mut rx = state.broadcast(ch)?.subscribe();

    // The stream body needs its own handle; the outer scope still reads
    // config for the keep-alive interval after the stream is built.
    let stream_state = state.clone();
    let stream = async_stream::stream! {
        // Initial hello event so the client knows the channel.
        yield Ok::<_, Infallible>(Event::default()
            .event("hello")
            .data(serde_json::json!({
                "channel": ch.id(),
                "member_id": member_id.to_string(),
                "role": claims.role,
            }).to_string()));

        // Heartbeat.
        let mut hb = tokio::time::interval(Duration::from_secs(stream_state.config().heartbeat_seconds));
        hb.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                ev = rx.recv() => {
                    match ev {
                        Ok(env) => {
                            if env.member_id != member_id {
                                continue;
                            }
                            yield Ok(Event::default()
                                .event(&env.event_name)
                                .id(env.event_id.to_string())
                                .data(serde_json::to_string(&env).unwrap_or_default()));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            yield Ok(Event::default()
                                .comment(format!("lagged {n}")));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = hb.tick() => {
                    // SSE comments double as keep-alives.
                    yield Ok(Event::default().comment("keep-alive"));
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new().interval(Duration::from_secs(state.config().heartbeat_seconds)),
    ))
}

#[derive(Serialize)]
pub struct _HelloPayload {
    pub channel: String,
    pub member_id: String,
    pub role: String,
}
