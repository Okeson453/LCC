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

use crate::domain::{InteractionDirection, InteractionKind};
use crate::error::Error;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/contacts", get(list_contacts).post(create_contact))
        .route("/api/v1/contacts/:id", get(get_contact))
        .route(
            "/api/v1/contacts/:id",
            axum::routing::patch(update_contact).delete(delete_contact),
        )
        .route(
            "/api/v1/contacts/:id/interactions",
            post(record_interaction),
        )
        .route(
            "/api/v1/contacts/:id/company",
            axum::routing::patch(link_company),
        )
        .route("/api/v1/companies", get(list_companies).post(create_company))
        .route("/api/v1/companies/staleness", get(staleness))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    limit: Option<i64>,
}

async fn list_contacts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let v = state
        .service()
        .list_contacts(m, query.q.as_deref(), query.limit.unwrap_or(100).min(500))
        .await?;
    Ok(Json(json!({"contacts":v})))
}

async fn get_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    Ok(Json(state.service().get_contact(m, id).await?))
}

#[derive(Deserialize)]
struct CreateContactRequest {
    full_name: String,
    title: Option<String>,
    headline: Option<String>,
    linkedin_url: Option<String>,
    email: Option<String>,
    company_id: Option<Uuid>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
}

async fn create_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateContactRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let c = state
        .service()
        .create_contact(
            m,
            &req.full_name,
            req.title.as_deref(),
            req.headline.as_deref(),
            req.linkedin_url.as_deref(),
            req.email.as_deref(),
            req.company_id,
            req.tags.unwrap_or_default(),
            req.notes.as_deref(),
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(c)))
}

#[derive(Deserialize)]
struct UpdateContactRequest {
    version: i32,
    full_name: Option<String>,
    title: Option<Option<String>>,
    headline: Option<Option<String>>,
    linkedin_url: Option<Option<String>>,
    email: Option<Option<String>>,
    notes: Option<Option<String>>,
    tags: Option<Vec<String>>,
    company_id: Option<Option<Uuid>>,
}

async fn update_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateContactRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let patch = crate::service::ContactPatch {
        full_name: req.full_name,
        title: req.title,
        headline: req.headline,
        linkedin_url: req.linkedin_url,
        email: req.email,
        notes: req.notes,
        tags: req.tags,
        company_id: req.company_id,
    };
    Ok(Json(
        state
            .service()
            .update_contact(m, id, req.version, patch)
            .await?,
    ))
}

async fn delete_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    state.service().delete_contact(m, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
struct RecordInteractionRequest {
    kind: String,
    summary: String,
    direction: Option<String>,
    channel: Option<String>,
    metadata: Option<Value>,
}

async fn record_interaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<RecordInteractionRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let kind = parse_kind(&req.kind)?;
    let direction = match req.direction.as_deref() {
        Some("inbound") => InteractionDirection::Inbound,
        Some("outbound") => InteractionDirection::Outbound,
        _ => InteractionDirection::Neutral,
    };
    let i = state
        .service()
        .record_interaction(
            m,
            id,
            kind,
            &req.summary,
            direction,
            req.channel,
            req.metadata.unwrap_or(Value::Null),
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(i)))
}

#[derive(Deserialize)]
struct LinkCompanyRequest {
    company_id: Uuid,
}

async fn link_company(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<LinkCompanyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    state.service().link(m, id, req.company_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}

async fn list_companies(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let v = state
        .service()
        .list_companies(m, query.q.as_deref(), query.limit.unwrap_or(100).min(500))
        .await?;
    Ok(Json(json!({"companies":v})))
}

#[derive(Deserialize)]
struct CreateCompanyRequest {
    name: String,
    domain: Option<String>,
    industry: Option<String>,
    size_band: Option<String>,
    funding_stage: Option<String>,
    tech_stack: Option<Vec<String>>,
    trigger_events: Option<Vec<String>>,
    public_signals: Option<Vec<String>>,
    ttl_at: Option<chrono::NaiveDate>,
}

async fn create_company(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCompanyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let c = state
        .service()
        .create_company(
            m,
            &req.name,
            req.domain.as_deref(),
            req.industry.as_deref(),
            req.size_band.as_deref(),
            req.funding_stage.as_deref(),
            req.tech_stack.unwrap_or_default(),
            req.trigger_events.unwrap_or_default(),
            req.public_signals.unwrap_or_default(),
            req.ttl_at,
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(c)))
}

#[derive(Deserialize)]
struct StalenessQuery {
    days: Option<i64>,
}

async fn staleness(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<StalenessQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = require_member(&headers)?;
    let r = state
        .service()
        .staleness(m, q.days.unwrap_or(60))
        .await?;
    Ok(Json(r))
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

fn parse_kind(s: &str) -> Result<InteractionKind, Error> {
    Ok(match s {
        "manual_note" => InteractionKind::ManualNote,
        "inbound_message" => InteractionKind::InboundMessage,
        "outbound_message" => InteractionKind::OutboundMessage,
        "connection_request" => InteractionKind::ConnectionRequest,
        "phone_call" => InteractionKind::PhoneCall,
        "meeting" => InteractionKind::Meeting,
        "sequence_step_sent" => InteractionKind::SequenceStepSent,
        "reaction" => InteractionKind::Reaction,
        "public_comment" => InteractionKind::PublicComment,
        "other" => InteractionKind::Other,
        other => return Err(Error::Validation(format!("unknown kind {other}"))),
    })
}
