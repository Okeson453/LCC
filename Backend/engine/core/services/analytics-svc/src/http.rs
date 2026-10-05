use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::Granularity;
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/analytics/dashboard", get(dashboard))
        .route("/api/v1/analytics/time-series", get(time_series))
        .with_state(state)
}

#[derive(Deserialize)]
struct DashboardQuery {
    start: NaiveDate,
    end: NaiveDate,
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<DashboardQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().dashboard(m, q.start, q.end).await?))
}

#[derive(Deserialize)]
struct SeriesQuery {
    metric: String,
    granularity: String,
    start: NaiveDate,
    end: NaiveDate,
}

async fn time_series(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<SeriesQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let granularity = match q.granularity.as_str() {
        "day" => Granularity::Day,
        "week" => Granularity::Week,
        "month" => Granularity::Month,
        other => return Err(Error::Validation(format!("unknown granularity {other}"))),
    };
    Ok(Json(
        state
            .service()
            .time_series(m, &q.metric, granularity, q.start, q.end)
            .await?,
    ))
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
