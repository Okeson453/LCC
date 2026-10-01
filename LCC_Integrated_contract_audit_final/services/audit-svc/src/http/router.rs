//! audit-svc HTTP router.

use axum::{
    routing::{delete, get, patch},
    Router,
};

use crate::{http::handlers, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::health::router())
        .route("/v1/audit_svc", get(handlers::root))
        .route(
            "/v1/audit_svc/items",
            get(handlers::list).post(handlers::create),
        )
        .route(
            "/v1/audit_svc/items/:id",
            get(handlers::get_one)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .with_state(state)
}
