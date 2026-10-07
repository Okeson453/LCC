//! Content-svc HTTP — canonical `/api/v1/members/{memberId}/content/*`
//! namespace, per `lcc-api-canonical.yaml`.
//!
//! The gateway forwards the path verbatim, so these literals must match the
//! contract exactly.

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
    // Plain string literals, not `format!`/`const` — the service-routing
    // conformance test reads these literals to prove the served paths match
    // the contract, so an indirection would hide the route from it.
    Router::new()
        .route("/api/v1/members/:member_id/content", get(list).post(create))
        .route(
            "/api/v1/members/:member_id/content/:content_id",
            get(get_one).patch(update_body).delete(delete_one),
        )
        .route(
            "/api/v1/members/:member_id/content/:content_id/quality-check",
            post(quality_check),
        )
        .route(
            "/api/v1/members/:member_id/content/:content_id/submit-for-approval",
            post(transition),
        )
        .route(
            "/api/v1/members/:member_id/content/:content_id/schedule",
            post(schedule),
        )
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    state: Option<String>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    let st = parse_state(q.state.as_deref())?;
    Ok(Json(
        json!({"items": state.service().list(m, st, q.limit.unwrap_or(50).min(200)).await?}),
    ))
}

async fn get_one(
    State(state): State<AppState>,
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    Ok(Json(state.service().get(m, content_id).await?))
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
    Path(member_id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<CreateRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
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
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(req): Json<UpdateBodyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    Ok(Json(
        state
            .service()
            .update_body(m, content_id, req.version, &req.body)
            .await?,
    ))
}

async fn delete_one(
    State(state): State<AppState>,
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    state.service().delete(m, content_id).await?;
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
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(req): Json<TransitionRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    let st = parse_state(Some(&req.new_state))?
        .ok_or_else(|| Error::Validation("new_state required".into()))?;
    Ok(Json(
        state
            .service()
            .transition(m, content_id, req.version, st, req.scheduled_at)
            .await?,
    ))
}

#[derive(Deserialize)]
struct QualityCheckRequest {
    version: i32,
}

async fn quality_check(
    State(state): State<AppState>,
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(req): Json<QualityCheckRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    let result = state
        .service()
        .run_quality_loop(m, content_id, req.version)
        .await?;
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
    Path((member_id, content_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(req): Json<ScheduleRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = member(&headers, member_id)?;
    let schedule = Schedule {
        scheduled_at: req.scheduled_at,
        slots: req.slots,
    };
    Ok(Json(
        state
            .service()
            .schedule(m, content_id, req.version, schedule)
            .await?,
    ))
}

/// Resolve the caller from the bearer token and refuse if the `{memberId}`
/// path segment names a different member. The JWT stays authoritative for who
/// is calling; a mismatch is a cross-member access attempt, not a no-op.
fn member(headers: &HeaderMap, member_id: Uuid) -> Result<Uuid, Error> {
    let m = require_member(headers)?;
    if m != member_id {
        return Err(Error::Forbidden);
    }
    Ok(m)
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
