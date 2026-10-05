use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/audit/events", get(list))
        .route("/api/v1/audit/events/:id", get(get_one))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    event_name: Option<String>,
    producer_service: Option<String>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state
        .service()
        .list(
            m,
            q.event_name.as_deref(),
            q.producer_service.as_deref(),
            q.start,
            q.end,
            q.limit.unwrap_or(100).min(500),
        )
        .await?;
    Ok(Json(json!({"events":v})))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let v = state.service().get(id).await?;
    Ok(Json(v))
}

fn auth(headers: &HeaderMap) -> Result<Uuid, Error> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(Error::Unauthorized)?;
    let claims = lcc_auth::verify_token(
        token,
        &std::env::var("LCC_AUTH_JWT_SECRET").map_err(|_| Error::Unauthorized)?,
    )
    .map_err(|_| Error::Unauthorized)?;
    Uuid::parse_str(&claims.sub).map_err(|_| Error::Unauthorized)
}
