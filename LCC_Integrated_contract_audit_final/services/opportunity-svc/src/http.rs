use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::domain::{OpportunityStatus, Source};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/opportunities", get(list).post(discover))
        .route("/api/v1/opportunities/:id/qualify", post(qualify))
        .route(
            "/api/v1/opportunities/:id/apply",
            axum::routing::patch(apply),
        )
        .route("/api/v1/opportunities/applications", get(list_applications))
        .route("/api/v1/opportunities/proposals", get(list_proposals).post(propose))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery { status: Option<String>, limit: Option<i64> }

async fn list(
    State(state): State<AppState>, headers: HeaderMap, Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let st = q.status.as_deref().map(parse_status).transpose()?;
    let v = state.service().list(m, st, q.limit.unwrap_or(50).min(200)).await?;
    Ok(Json(json!({"opportunities":v})))
}

#[derive(Deserialize)]
struct DiscoverRequest {
    title: String,
    source: String,
    company_id: Option<Uuid>,
    metadata: Option<Value>,
}

async fn discover(
    State(state): State<AppState>, headers: HeaderMap, Json(req): Json<DiscoverRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let src = parse_source(&req.source)?;
    let o = state.service().discover(m, &req.title, src, req.company_id, req.metadata.unwrap_or(Value::Null)).await?;
    Ok((axum::http::StatusCode::CREATED, Json(o)))
}

#[derive(Deserialize)]
struct QualifyRequest {
    version: i32,
    fit_score: f64,
    next_state: String,
}

async fn qualify(
    State(state): State<AppState>, Path(id): Path<Uuid>, headers: HeaderMap, Json(req): Json<QualifyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let ns = parse_status(&req.next_state)?
        .ok_or_else(|| Error::Validation("next_state required".into()))?;
    Ok(Json(state.service().qualify(m, id, req.version, req.fit_score, ns).await?))
}

#[derive(Deserialize)]
struct ApplyRequest {
    cover_letter: Option<String>,
    resume_doc_id: Option<Uuid>,
}

async fn apply(
    State(state): State<AppState>, Path(id): Path<Uuid>, headers: HeaderMap, Json(req): Json<ApplyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let a = state.service().apply(m, id, req.cover_letter, req.resume_doc_id).await?;
    Ok((axum::http::StatusCode::CREATED, Json(a)))
}

async fn list_applications(
    State(state): State<AppState>, headers: HeaderMap, Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state.service().list_applications(m, q.limit.unwrap_or(50).min(200)).await?;
    Ok(Json(json!({"applications":v})))
}

#[derive(Deserialize)]
struct ProposeRequest {
    opportunity_id: Uuid,
    title: String,
    body: String,
    price_cents: Option<i64>,
    currency: Option<String>,
    kb_ref_ids: Vec<Uuid>,
}

async fn propose(
    State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ProposeRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let p = state.service().propose(m, req.opportunity_id, &req.title, &req.body,
        req.price_cents, req.currency, req.kb_ref_ids).await?;
    Ok((axum::http::StatusCode::CREATED, Json(p)))
}

async fn list_proposals(
    State(state): State<AppState>, headers: HeaderMap, Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state.service().list_proposals(m, q.limit.unwrap_or(50).min(200)).await?;
    Ok(Json(json!({"proposals":v})))
}

fn auth(headers: &HeaderMap) -> Result<Uuid, Error> {
    let token = headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(Error::Unauthorized)?;
    let claims = lcc_auth::verify_token(token, &std::env::var("LCC_AUTH_JWT_SECRET").map_err(|_| Error::Unauthorized)?)
        .map_err(|_| Error::Unauthorized)?;
    Uuid::parse_str(&claims.sub).map_err(|_| Error::Unauthorized)
}

fn parse_status(s: &str) -> Result<Option<OpportunityStatus>, Error> {
    Ok(Some(match s {
        "discovered" => OpportunityStatus::Discovered,
        "qualified" => OpportunityStatus::Qualified,
        "contacted" => OpportunityStatus::Contacted,
        "conversation" => OpportunityStatus::Conversation,
        "applied" => OpportunityStatus::Applied,
        "interviewing" => OpportunityStatus::Interviewing,
        "offer" => OpportunityStatus::Offer,
        "rejected" => OpportunityStatus::Rejected,
        "closed" => OpportunityStatus::Closed,
        "withdrawn" => OpportunityStatus::Withdrawn,
        "" => return Ok(None),
        other => return Err(Error::Validation(format!("unknown status {other}"))),
    }))
}

fn parse_source(s: &str) -> Result<Source, Error> {
    Ok(match s {
        "manual" => Source::Manual,
        "inbound" => Source::Inbound,
        "linkedin_search" => Source::LinkedInSearch,
        "job_board" => Source::JobBoard,
        "referral" => Source::Referral,
        other => return Err(Error::Validation(format!("unknown source {other}"))),
    })
}

#[allow(dead_code)] fn _t(_: Value) {}
