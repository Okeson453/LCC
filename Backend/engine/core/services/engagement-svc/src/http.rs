use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use lcc_auth::{rbac::Permission, Caller};
use serde::Deserialize;
use serde_json::json;
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
        // F-AUDIT-56: this merge was missing, so the service exposed no
        // health endpoint at all and its pod would have crash-looped on the
        // manifest's /healthz liveness probe.
        //
        // Merged AFTER `.with_state()`: the health router is stateless, so
        // merging it first would pin the router's state type to `()` and every
        // stateful route below would stop type-checking.
        .merge(crate::health::router())
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
    let c = auth(&headers)?;
    // Reading one's own inbox is a `ViewOwnData` read, which every role holds.
    require(c, Permission::ViewOwnData)?;
    let limit = clamp_limit(q.limit);
    // One extra row answers `has_more` honestly instead of guessing from a
    // full page; the contract's `InboxPage` carries the flag.
    let mut v = state
        .service()
        .inbox(c.member_id, limit + 1, q.unread_only.unwrap_or(false))
        .await?;
    let has_more = v.len() > limit as usize;
    v.truncate(limit as usize);
    Ok(Json(json!({
        "items": v,
        "next_cursor": serde_json::Value::Null,
        "has_more": has_more,
    })))
}

async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let c = auth(&headers)?;
    require(c, Permission::ViewOwnData)?;
    // 204 whether or not a row matched: an unknown id and another member's id
    // must be indistinguishable.
    state.service().mark_read(c.member_id, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let c = auth(&headers)?;
    require(c, Permission::ViewOwnData)?;
    let status = q.status.as_deref().map(parse_status).transpose()?;
    let v = state
        .service()
        .list(c.member_id, status, clamp_limit(q.limit))
        .await?;
    // The contract declares this 200 as a bare array of EngagementTask
    // (`/members/{memberId}/engagement/queue`), and the dashboard client types
    // it as `QueueItem[]`. The previous `{"tasks": [...]}` envelope matched
    // neither.
    Ok(Json(v))
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
    let c = auth(&headers)?;
    // Creating a unit of engagement work is drafting, so it takes the same
    // permission as drafting content: an Auditor (read-only) and a Reviewer
    // (approve-only) may not add to the queue.
    require(c, Permission::DraftContent)?;
    let a = parse_action(&req.action_type)?;
    let t = state
        .service()
        .create_task(
            c.member_id,
            a,
            req.contact_id,
            req.target_post_id,
            req.priority_score,
            req.due_at,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(t)))
}

#[derive(Deserialize)]
struct DraftRequest {
    version: i32,
    /// design §11.12 `draft_body` / contract `EngagementTask.draft_body`.
    /// `draft_pins` is intentionally not accepted: no column, no contract
    /// field, no design field.
    draft_body: String,
}

async fn update_draft(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<DraftRequest>,
) -> Result<impl IntoResponse, Error> {
    let c = auth(&headers)?;
    require(c, Permission::EditContent)?;
    Ok(Json(
        state
            .service()
            .update_draft(c.member_id, id, req.version, &req.draft_body)
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
    let c = auth(&headers)?;
    // `complete` is the send/approve transition on an outbound reply. The
    // contract calls it Tier-2 ("Approve a reply/comment/congratulation send"),
    // so it needs a send-capable role, not merely a valid token.
    require(c, Permission::ApproveSend)?;
    Ok(Json(
        state
            .service()
            .complete(c.member_id, id, req.version)
            .await?,
    ))
}

async fn dismiss(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<VersionedRequest>,
) -> Result<impl IntoResponse, Error> {
    let c = auth(&headers)?;
    require(c, Permission::EditContent)?;
    Ok(Json(
        state
            .service()
            .dismiss(c.member_id, id, req.version)
            .await?,
    ))
}

/// Bound `limit` to the contract's page size. A negative or zero limit is
/// coerced rather than passed through, because `LIMIT 0` would answer 200 with
/// an empty queue that looks like "you have no work".
fn clamp_limit(limit: Option<i64>) -> i64 {
    match limit {
        Some(n) if n > 0 => n.min(200),
        Some(_) => 50,
        None => 50,
    }
}

/// Verify the bearer token and return the caller's member id and role.
///
/// `member_id` comes from the verified `sub` claim and never from a header, path
/// or body value, so a caller cannot reach another member's rows by supplying a
/// different id. `lcc_auth::caller_from_headers` fails closed: a missing,
/// non-`Bearer`, badly-signed, expired or non-UUID-`sub` credential is 401, and
/// so is a token whose `role` is not one of the five known roles.
fn auth(headers: &HeaderMap) -> Result<Caller, Error> {
    lcc_auth::caller_from_headers(headers).map_err(from_lcc_error)
}

/// Require a permission, keeping 401 and 403 distinct: an unusable credential
/// is 401, an authenticated caller without the right is 403 (RFC 9110).
fn require(caller: Caller, perm: Permission) -> Result<(), Error> {
    caller.require(perm).map_err(from_lcc_error)
}

fn from_lcc_error(e: lcc_auth::LccError) -> Error {
    match e {
        lcc_auth::LccError::Unauthorized(_) => Error::Unauthorized,
        lcc_auth::LccError::Forbidden(_) => Error::Forbidden,
        lcc_auth::LccError::Validation(m) => Error::Validation(m),
        other => Error::Internal(other.to_string()),
    }
}

/// design §11.12's CHECK list == the contract enum == the proto enum.
/// The previous parser also accepted `completed` and `skipped`, neither of
/// which is in any authority and both of which the column now rejects.
fn parse_action(s: &str) -> Result<ActionType, Error> {
    s.parse()
        .map_err(|_| Error::Validation(format!("unknown action_type {s:?}")))
}

fn parse_status(s: &str) -> Result<TaskStatus, Error> {
    s.parse()
        .map_err(|_| Error::Validation(format!("unknown status {s:?}")))
}
