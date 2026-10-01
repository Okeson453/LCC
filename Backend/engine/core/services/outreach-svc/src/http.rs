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

use crate::domain::{SequenceStatus, StepKind, TemplateStep};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/sequences", get(list).post(create))
        .route("/api/v1/sequences/:id/steps", get(steps))
        .route(
            "/api/v1/sequences/:id/pause",
            axum::routing::patch(pause),
        )
        .route(
            "/api/v1/sequences/:id/resume",
            post(resume),
        )
        .route(
            "/api/v1/sequences/steps/:step_id/sent",
            post(mark_sent),
        )
        .route(
            "/api/v1/sequences/steps/:step_id/reply",
            post(mark_reply),
        )
        .route("/api/v1/outreach/templates", get(list_templates).post(create_template))
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
    let st = q.status.as_deref().map(parse_status).transpose()?;
    let v = state
        .service()
        .list_sequences(m, st, q.limit.unwrap_or(50).min(200))
        .await?;
    Ok(Json(json!({"sequences":v})))
}

#[derive(Deserialize)]
struct CreateRequest {
    contact_id: Uuid,
    template_id: Option<Uuid>,
    steps: Vec<TemplateStep>,
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let s = state
        .service()
        .create_sequence(m, req.contact_id, req.template_id, req.steps)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(s)))
}

async fn steps(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let v = state.service().list_steps(id).await?;
    Ok(Json(json!({"steps":v})))
}

#[derive(Deserialize)]
struct PauseRequest {
    version: i32,
    reason: String,
}

async fn pause(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<PauseRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().pause(m, id, req.version, &req.reason).await?))
}

#[derive(Deserialize)]
struct VersionedRequest {
    version: i32,
}

async fn resume(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<VersionedRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().resume(m, id, req.version).await?))
}

async fn mark_sent(
    State(state): State<AppState>,
    Path(step_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    state.service().mark_step_sent(step_id, m).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

async fn mark_reply(
    State(state): State<AppState>,
    Path(step_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    state.service().mark_step_reply(step_id, m).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(json!({"templates": state.service().list_templates(m).await?})))
}

#[derive(Deserialize)]
struct CreateTemplateRequest {
    name: String,
    description: Option<String>,
    steps: Vec<TemplateStep>,
}

async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let t = state
        .service()
        .create_template(m, &req.name, req.description.as_deref(), req.steps)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(t)))
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

fn parse_status(s: &str) -> Result<SequenceStatus, Error> {
    Ok(match s {
        "draft" => SequenceStatus::Draft,
        "active" => SequenceStatus::Active,
        "paused" => SequenceStatus::Paused,
        "completed" => SequenceStatus::Completed,
        "cancelled" => SequenceStatus::Cancelled,
        "failed" => SequenceStatus::Failed,
        other => return Err(Error::Validation(format!("unknown status {other}"))),
    })
}

#[allow(dead_code)] fn _t(_: Value) {}
#[allow(dead_code)] fn _k(_: StepKind) {}
