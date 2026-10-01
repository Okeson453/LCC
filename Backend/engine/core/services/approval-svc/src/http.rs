use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{ApprovalStatus, BulkDecideInput, RequestApprovalInput};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/approvals", get(list).post(request))
        .route("/api/v1/approvals/:id", get(get_one))
        .route(
            "/api/v1/approvals/:id/decide",
            axum::routing::patch(decide_one),
        )
        .route("/api/v1/approvals/bulk-decide", post(bulk_decide))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let st = q
        .status
        .as_deref()
        .map(parse_status)
        .transpose()?;
    let v = state
        .service()
        .list(m, st, q.limit.unwrap_or(50).min(200))
        .await?;
    Ok(Json(serde_json::json!({"approvals":v})))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().get(m, id).await?))
}

async fn request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RequestApprovalInput>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let a = state.service().request(m, m, input).await?;
    Ok((axum::http::StatusCode::CREATED, Json(a)))
}

#[derive(Deserialize)]
struct DecideRequest {
    version: i32,
    decision: String,
    reason: Option<String>,
}

async fn decide_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<DecideRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let new_status = parse_status(&req.decision)?;
    let a = state
        .service()
        .decide(m, m, id, new_status, req.reason.as_deref(), req.version)
        .await?;
    Ok(Json(a))
}

async fn bulk_decide(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BulkDecideInput>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let r = state.service().bulk_decide(m, m, input).await?;
    Ok(Json(r))
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

fn parse_status(s: &str) -> Result<ApprovalStatus, Error> {
    Ok(match s {
        "pending" => ApprovalStatus::Pending,
        "approved" => ApprovalStatus::Approved,
        "rejected" => ApprovalStatus::Rejected,
        "expired" => ApprovalStatus::Expired,
        "cancelled" => ApprovalStatus::Cancelled,
        other => return Err(Error::Validation(format!("unknown status {other}"))),
    })
}

#[allow(dead_code)]
fn _t(_: Value) {}
