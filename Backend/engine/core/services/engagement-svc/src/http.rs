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

use crate::domain::{ActionType, TaskStatus};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/engagement/inbox", get(inbox))
        .route(
            "/api/v1/engagement/inbox/:id/read",
            axum::routing::post(mark_read),
        )
        .route("/api/v1/engagement/queue", get(queue))
        .route("/api/v1/engagement/tasks", get(queue).post(create_task))
        .route(
            "/api/v1/engagement/tasks/:id/draft",
            axum::routing::patch(update_draft),
        )
        .route("/api/v1/engagement/tasks/:id/complete", post(complete))
        .route("/api/v1/engagement/tasks/:id/dismiss", post(dismiss))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct InboxQuery {
    unread_only: Option<bool>,
    limit: Option<i64>,
}

async fn inbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<InboxQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state
        .service()
        .inbox(
            m,
            q.limit.unwrap_or(50).min(200),
            q.unread_only.unwrap_or(false),
        )
        .await?;
    Ok(Json(json!({"messages":v})))
}

async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    state.service().mark_read(m, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let status = q.status.as_deref().map(parse_status).transpose()?;
    let v = state
        .service()
        .list(m, status, q.limit.unwrap_or(50).min(200))
        .await?;
    Ok(Json(json!({"tasks":v})))
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    action_type: String,
    contact_id: Option<Uuid>,
    target_post_id: Option<String>,
    priority_score: Option<f64>,
    due_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let a = parse_action(&req.action_type)?;
    let t = state
        .service()
        .create_task(
            m,
            a,
            req.contact_id,
            req.target_post_id,
            req.priority_score,
            req.due_at,
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(t)))
}

#[derive(Deserialize)]
struct DraftRequest {
    version: i32,
    draft: String,
    draft_pins: Vec<Uuid>,
}

async fn update_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<DraftRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(
        state
            .service()
            .update_draft(m, id, req.version, &req.draft, &req.draft_pins)
            .await?,
    ))
}

#[derive(Deserialize)]
struct VersionedRequest {
    version: i32,
}

async fn complete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<VersionedRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().complete(m, id, req.version).await?))
}

async fn dismiss(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<VersionedRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().dismiss(m, id, req.version).await?))
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

fn parse_action(s: &str) -> Result<ActionType, Error> {
    Ok(match s {
        "reply" => ActionType::Reply,
        "comment" => ActionType::Comment,
        "like" => ActionType::Like,
        "connect" => ActionType::Connect,
        "remind" => ActionType::Remind,
        "share" => ActionType::Share,
        "publish" => ActionType::Publish,
        "custom_note" => ActionType::CustomNote,
        other => return Err(Error::Validation(format!("unknown action {other}"))),
    })
}

fn parse_status(s: &str) -> Result<TaskStatus, Error> {
    Ok(match s {
        "queued" => TaskStatus::Queued,
        "drafted" => TaskStatus::Drafted,
        "sent" => TaskStatus::Sent,
        "completed" => TaskStatus::Completed,
        "skipped" => TaskStatus::Skipped,
        "expired" => TaskStatus::Expired,
        other => return Err(Error::Validation(format!("unknown status {other}"))),
    })
}

#[allow(dead_code)]
fn _t(_: Value) {}
