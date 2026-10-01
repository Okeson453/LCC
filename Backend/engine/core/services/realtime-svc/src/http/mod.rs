//! HTTP router for the realtime service.

use axum::{routing::get, Router};

use crate::{sse, state::AppState, ws};

pub fn build_router(state: AppState) -> Router {
    // Sub-routers share `AppState`; `.with_state(state)` erases it at the end.
    Router::<AppState>::new()
        .merge(crate::health::router())
        // Canonical WS channel routes.
        .route("/api/v1/ws/briefing", get(ws::ws_handler))
        .route("/api/v1/ws/approvals", get(ws::ws_handler))
        .route("/api/v1/ws/engagement", get(ws::ws_handler))
        .route("/api/v1/ws/compliance", get(ws::ws_handler))
        .route("/api/v1/ws/sequence", get(ws::ws_handler))
        // Catch-all for any other future channels.
        .route("/api/v1/ws/:channel", get(ws::ws_handler))
        // Canonical SSE fallback.
        .route("/api/v1/sse/briefing", get(sse::sse_handler))
        .route("/api/v1/sse/approvals", get(sse::sse_handler))
        .route("/api/v1/sse/engagement", get(sse::sse_handler))
        .route("/api/v1/sse/compliance", get(sse::sse_handler))
        .route("/api/v1/sse/sequence", get(sse::sse_handler))
        .route("/api/v1/sse/:channel", get(sse::sse_handler))
        .with_state(state)
}
