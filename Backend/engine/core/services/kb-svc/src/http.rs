//! KB-svc HTTP handlers and router.

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

use crate::domain::{EmbeddingStatus, KbKind};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/kb/records", get(list).post(create))
        .route("/api/v1/kb/records/:id", get(get_one))
        .route("/api/v1/kb/records/:id", axum::routing::patch(update))
        .route("/api/v1/kb/records/:id", axum::routing::delete(delete))
        .route("/api/v1/kb/records/:id/reembed", post(reembed))
        .route(
            "/api/v1/kb/records/:id/embedding-status",
            post(set_status),
        )
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    kind: Option<String>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    let kind = parse_kind(q.kind.as_deref())?;
    let records = state
        .service()
        .list(member, kind, q.limit.unwrap_or(50).min(200))
        .await?;
    Ok(Json(json!({"records":records})))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    let record = state.service().get(member, id).await?;
    Ok(Json(record))
}

#[derive(Deserialize)]
struct CreateRequest {
    kind: String,
    title: String,
    body: String,
    tags: Vec<String>,
    source: Option<String>,
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRequest>,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    // A record must have a kind; the list filter at the top of this file is
    // the only caller where "any kind" is meaningful.
    let kind = parse_kind(Some(req.kind.as_str()))?
        .ok_or_else(|| Error::Validation("kind is required".to_string()))?;
    let r = state
        .service()
        .create(member, kind, &req.title, &req.body, req.tags, req.source)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(r)))
}

#[derive(Deserialize)]
struct UpdateRequest {
    version: i32,
    title: String,
    body: String,
    tags: Vec<String>,
    source: Option<String>,
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateRequest>,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    let r = state
        .service()
        .update(
            member,
            id,
            req.version,
            &req.title,
            &req.body,
            &req.tags,
            req.source.as_deref(),
        )
        .await?;
    Ok(Json(r))
}

async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    state.service().delete(member, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

async fn reembed(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let member = require_member(&headers)?;
    let v = state.service().reembed(member, id).await?;
    Ok(Json(json!({"id":id,"version":v})))
}

#[derive(Deserialize)]
struct SetStatusRequest {
    status: String,
    embedding_id: Option<String>,
}

async fn set_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<SetStatusRequest>,
) -> Result<impl IntoResponse, Error> {
    // Internal endpoint — worker-only. Caller check via service token.
    let member = require_member(&headers)?;
    let status = parse_status(&req.status)?;
    state
        .service()
        .set_embedding_status(id, status, req.embedding_id)
        .await?;
    let _ = member; // unused after auth
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

fn require_member(headers: &HeaderMap) -> Result<Uuid, Error> {
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

fn parse_kind(s: Option<&str>) -> Result<Option<KbKind>, Error> {
    let Some(s) = s else { return Ok(None) };
    let v = match s {
        "experience" => KbKind::Experience,
        "skill" => KbKind::Skill,
        "project" => KbKind::Project,
        "principle" => KbKind::Principle,
        "voice_style" => KbKind::VoiceStyle,
        "industry" => KbKind::Industry,
        "persona" => KbKind::Persona,
        other => return Err(Error::Validation(format!("unknown kind {other}"))),
    };
    Ok(Some(v))
}

fn parse_status(s: &str) -> Result<EmbeddingStatus, Error> {
    Ok(match s {
        "pending" => EmbeddingStatus::Pending,
        "embedded" => EmbeddingStatus::Embedded,
        "failed" => EmbeddingStatus::Failed,
        "stale" => EmbeddingStatus::Stale,
        other => return Err(Error::Validation(format!("unknown status {other}"))),
    })
}

#[allow(dead_code)]
fn _unused(_: Value) {}
