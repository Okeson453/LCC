//! Content-svc HTTP — canonical /api/v1/content/* namespace.

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

use crate::domain::{ContentKind, ContentState, Schedule};
use crate::error::Error;
use crate::service::NewContentItem;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/content/items", get(list).post(create))
        .route("/api/v1/content/items/:id", get(get_one))
        .route(
            "/api/v1/content/items/:id",
            axum::routing::patch(update_body).delete(delete_one),
        )
        .route("/api/v1/content/items/:id/transition", post(transition))
        .route(
            "/api/v1/content/items/:id/quality-check",
            post(quality_check),
        )
        .route("/api/v1/content/items/:id/schedule", post(schedule))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    state: Option<String>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let st = parse_state(q.state.as_deref())?;
    Ok(Json(
        json!({"items": state.service().list(m, st, q.limit.unwrap_or(50).min(200)).await?}),
    ))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    Ok(Json(state.service().get(m, id).await?))
}

#[derive(Deserialize)]
struct CreateRequest {
    title: String,
    body: String,
    kind: String,
    topic: String,
    voice_style_kb_id: Option<Uuid>,
    pinned_kb_ids: Option<Vec<Uuid>>,
    idempotency_key: Option<String>,
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let kind = parse_kind(&req.kind)?;
    let item = state
        .service()
        .create(
            m,
            NewContentItem {
                title: &req.title,
                body: &req.body,
                kind,
                topic: &req.topic,
                voice_style_kb_id: req.voice_style_kb_id,
                pinned_kb_ids: req.pinned_kb_ids.unwrap_or_default(),
                idempotency_key: req.idempotency_key,
            },
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

#[derive(Deserialize)]
struct UpdateBodyRequest {
    version: i32,
    body: String,
}

async fn update_body(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateBodyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    Ok(Json(
        state
            .service()
            .update_body(m, id, req.version, &req.body)
            .await?,
    ))
}

async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    state.service().delete(m, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
struct TransitionRequest {
    version: i32,
    new_state: String,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn transition(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<TransitionRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let st = parse_state(Some(&req.new_state))?
        .ok_or_else(|| Error::Validation("new_state required".into()))?;
    Ok(Json(
        state
            .service()
            .transition(m, id, req.version, st, req.scheduled_at)
            .await?,
    ))
}

#[derive(Deserialize)]
struct QualityCheckRequest {
    version: i32,
}

async fn quality_check(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<QualityCheckRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let result = state.service().run_quality_loop(m, id, req.version).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ScheduleRequest {
    version: i32,
    scheduled_at: chrono::NaiveDateTime,
    slots: Vec<String>,
}

async fn schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<ScheduleRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let schedule = Schedule {
        scheduled_at: req.scheduled_at,
        slots: req.slots,
    };
    Ok(Json(
        state
            .service()
            .schedule(m, id, req.version, schedule)
            .await?,
    ))
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

fn parse_state(s: Option<&str>) -> Result<Option<ContentState>, Error> {
    let Some(s) = s else { return Ok(None) };
    Ok(Some(match s {
        "idea" => ContentState::Idea,
        "drafted" => ContentState::Drafted,
        "in_review" => ContentState::InReview,
        "approved" => ContentState::Approved,
        "scheduled" => ContentState::Scheduled,
        "published" => ContentState::Published,
        "rejected" => ContentState::Rejected,
        "publish_failed" => ContentState::PublishFailed,
        "blocked" => ContentState::Blocked,
        other => return Err(Error::Validation(format!("unknown state {other}"))),
    }))
}

fn parse_kind(s: &str) -> Result<ContentKind, Error> {
    Ok(match s {
        "post" => ContentKind::Post,
        "article" => ContentKind::Article,
        "comment" => ContentKind::Comment,
        "reply" => ContentKind::Reply,
        "sequence_message" => ContentKind::SequenceMessage,
        other => return Err(Error::Validation(format!("unknown kind {other}"))),
    })
}

#[allow(dead_code)]
fn _unused(_: Value) {}
