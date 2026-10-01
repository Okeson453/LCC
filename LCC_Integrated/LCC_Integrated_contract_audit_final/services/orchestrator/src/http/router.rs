//! orchestrator HTTP router.

use axum::{
    routing::{delete, get, patch},
    Router,
};

use crate::{http::handlers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::health::router())
        .route("/v1/orchestrator", get(handlers::root))
        .route(
            "/v1/orchestrator/items",
            get(handlers::list).post(handlers::create),
        )
        .route(
            "/v1/orchestrator/items/:id",
            get(handlers::get_one)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .with_state(state)
}
