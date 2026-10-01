//! Profile-svc HTTP handlers and router.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::domain::{ConsentKind, EditDraftStatus};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/profile/me", get(get_me))
        .route("/api/v1/profile/me", axum::routing::put(put_me))
        .route("/api/v1/profile/me/edit-drafts", post(create_draft))
        .route(
            "/api/v1/profile/me/edit-drafts/:draft_id/decide",
            axum::routing::patch(decide_draft),
        )
        .route(
            "/api/v1/profile/me/consent/:kind",
            get(get_consent).post(grant_consent),
        )
        .with_state(state)
        .route("/health", get(health))
}

async fn health() -> Json<Value> {
    json!({"status":"ok"})
}

#[derive(Clone, Copy)]
struct AuthedMember(pub Uuid);

async fn auth_member(headers: axum::http::HeaderMap) -> Result<AuthedMember, Error> {
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
    let member_id = Uuid::parse_str(&claims.sub).map_err(|_| Error::Unauthorized)?;
    Ok(AuthedMember(member_id))
}

async fn get_me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    let snap = state.service().get_profile(member_id).await?;
    Ok(Json(snap))
}

async fn put_me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(snap): Json<crate::domain::ProfileSnapshot>,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    let saved = state.service().save_snapshot(member_id, snap).await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

#[derive(Deserialize)]
struct CreateDraftRequest {
    profile_id: Uuid,
    proposed_fields: Value,
}

async fn create_draft(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateDraftRequest>,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    let draft = state
        .service()
        .create_edit_draft(member_id, req.profile_id, req.proposed_fields)
        .await?;
    Ok((StatusCode::CREATED, Json(draft)))
}

#[derive(Deserialize)]
struct DecideDraftRequest {
    decision: EditDraftStatus,
    version: i32,
}

async fn decide_draft(
    State(state): State<AppState>,
    Path(draft_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    Json(req): Json<DecideDraftRequest>,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    state
        .service()
        .decide_edit_draft(member_id, draft_id, req.decision, req.version)
        .await?;
    Ok((StatusCode::NO_CONTENT, ()))
}

async fn get_consent(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    let kind = parse_kind(&kind)?;
    let c = state.service().get_consent(member_id, kind).await?;
    Ok(Json(c))
}

#[derive(Deserialize)]
struct GrantConsentRequest {
    ttl_days: Option<i64>,
}

async fn grant_consent(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    headers: axum::http::HeaderMap,
    Json(req): Json<GrantConsentRequest>,
) -> Result<impl IntoResponse, Error> {
    let AuthedMember(member_id) = auth_member(headers).await?;
    let kind = parse_kind(&kind)?;
    let c = state
        .service()
        .grant_consent(member_id, kind, req.ttl_days)
        .await?;
    Ok((StatusCode::CREATED, Json(c)))
}

fn parse_kind(s: &str) -> Result<ConsentKind, Error> {
    match s {
        "linkedin_automation" => Ok(ConsentKind::LinkedinAutomation),
        "data_export" => Ok(ConsentKind::DataExport),
        "kb_personalization" => Ok(ConsentKind::KbPersonalization),
        other => Err(Error::Validation(format!("unknown consent kind: {other}"))),
    }
}
