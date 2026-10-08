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

/// Canonical approval namespace, member-scoped per
/// `lcc-api-canonical.yaml`.
///
/// The gateway forwards `/api/v1/members/{memberId}/approvals/...` verbatim to
/// this service (see the subdomain override in `api-gateway/src/proxy`), so
/// these paths must match the contract exactly.
///
/// `/bulk-decide` is registered before `/:approval_id` so the static segment
/// cannot be shadowed by the parameter.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/v1/members/:member_id/approvals",
            get(list).post(request),
        )
        .route(
            "/api/v1/members/:member_id/approvals/bulk-decide",
            post(bulk_decide),
        )
        .route(
            "/api/v1/members/:member_id/approvals/:approval_id",
            get(get_one),
        )
        .route(
            "/api/v1/members/:member_id/approvals/:approval_id/decide",
            post(decide_one),
        )
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

/// Resolve the acting member from the bearer token and reject the request if
/// the `{memberId}` path segment names a different member.
///
/// The contract puts `memberId` in the path; the JWT remains the source of
/// truth for *who is calling*, so a mismatch means the caller is asking for
/// someone else's approvals and is refused rather than silently served.
fn auth(headers: &HeaderMap, member_id: Uuid) -> Result<Uuid, Error> {
    let m = subject(headers)?;
    if m != member_id {
        return Err(Error::Forbidden);
    }
    Ok(m)
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    limit: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers, member_id)?;
    let st = q.status.as_deref().map(parse_status).transpose()?;
    let v = state
        .service()
        .list(m, st, q.limit.unwrap_or(50).min(200))
        .await?;
    Ok(Json(serde_json::json!({"approvals":v})))
}

async fn get_one(
    State(state): State<AppState>,
    Path((member_id, approval_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers, member_id)?;
    Ok(Json(state.service().get(m, approval_id).await?))
}

async fn request(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
    headers: HeaderMap,
    Json(input): Json<RequestApprovalInput>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers, member_id)?;
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
    Path((member_id, approval_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(req): Json<DecideRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers, member_id)?;
    let new_status = parse_status(&req.decision)?;
    let a = state
        .service()
        .decide(
            m,
            m,
            approval_id,
            new_status,
            req.reason.as_deref(),
            req.version,
        )
        .await?;
    Ok(Json(a))
}

async fn bulk_decide(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
    headers: HeaderMap,
    Json(input): Json<BulkDecideInput>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers, member_id)?;
    let r = state.service().bulk_decide(m, m, input).await?;
    Ok(Json(r))
}

fn subject(headers: &HeaderMap) -> Result<Uuid, Error> {
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
