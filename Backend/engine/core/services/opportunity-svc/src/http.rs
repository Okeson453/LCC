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

use crate::domain::{OpportunityCursor, OpportunityKind, OpportunityStatus, Source};
use crate::error::Error;
use crate::service::clamp_limit;
use crate::state::AppState;

/// Routes.
///
/// The namespace is still the flat `/api/v1/opportunities/…` form. That is
/// finding R-02 in `LCC_CODE_AUDIT_REPORT.md` §10 — the canonical contract
/// nests these under `/api/v1/members/{memberId}/…` and the api-gateway route
/// table would have to move in the same change. It is a repo-wide migration
/// touching the gateway and four other services, so it is deliberately left to
/// that change; what matters here is that the gateway still routes
/// `/api/v1/opportunities/*path` here verbatim, so these paths are live.
///
/// Authorization is the same on every route: the bearer token's `sub` claim
/// *is* the tenant. There is no `{memberId}` segment to disagree with, so
/// there is no way to ask for another member's data, and no member id is ever
/// taken from the request body or query string.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/opportunities", get(list).post(discover))
        .route("/api/v1/opportunities/applications", get(list_applications))
        .route(
            "/api/v1/opportunities/proposals",
            get(list_proposals).post(propose),
        )
        .route("/api/v1/opportunities/:id", get(get_one))
        .route("/api/v1/opportunities/:id/qualify", post(qualify))
        .route(
            "/api/v1/opportunities/:id/apply",
            axum::routing::patch(apply),
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

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    limit: Option<i64>,
    cursor: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let st = q.status.as_deref().map(parse_status).transpose()?.flatten();
    let cursor =
        match q.cursor.as_deref() {
            None => None,
            Some(raw) => Some(OpportunityCursor::decode(raw).ok_or_else(|| {
                Error::Validation("cursor is not a valid opportunity cursor".into())
            })?),
        };
    // `components.schemas.OpportunityPage` — `{items, next_cursor, has_more}`.
    let page = state
        .service()
        .list(m, st, clamp_limit(q.limit), cursor)
        .await?;
    Ok(Json(page))
}

#[derive(Deserialize)]
struct DiscoverRequest {
    title: String,
    /// The opportunity CATEGORY. Optional; defaults to `other`.
    kind: Option<String>,
    source: String,
    company_id: Option<Uuid>,
    contact_id: Option<Uuid>,
    company: Option<String>,
    metadata: Option<Value>,
}

async fn discover(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DiscoverRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let src = parse_source(&req.source)?;
    let kind = req
        .kind
        .as_deref()
        .map(|s| {
            OpportunityKind::parse(s)
                .ok_or_else(|| Error::Validation(format!("unknown opportunity kind {s}")))
        })
        .transpose()?;
    let o = state
        .service()
        .discover(
            m,
            &req.title,
            kind,
            src,
            req.company_id,
            req.contact_id,
            req.company,
            req.metadata.unwrap_or(Value::Null),
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(o)))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    Ok(Json(state.service().get(m, id).await?))
}

#[derive(Deserialize)]
struct QualifyRequest {
    version: i32,
    fit_score: f64,
    next_state: String,
    /// φ breakdown, stored in `lcc.opportunities.phi_components` and returned
    /// as the contract's `fit_components`.
    fit_components: Option<Value>,
}

async fn qualify(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<QualifyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let ns = parse_status(&req.next_state)?
        .ok_or_else(|| Error::Validation("next_state required".into()))?;
    Ok(Json(
        state
            .service()
            .qualify(
                m,
                id,
                req.version,
                req.fit_score,
                ns,
                req.fit_components.unwrap_or_else(|| json!({})),
            )
            .await?,
    ))
}

#[derive(Deserialize)]
struct ApplyRequest {
    cover_letter: Option<String>,
    resume_doc_id: Option<Uuid>,
}

async fn apply(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<ApplyRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let a = state
        .service()
        .apply(m, id, req.cover_letter, req.resume_doc_id)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(a)))
}

async fn list_applications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state
        .service()
        .list_applications(m, clamp_limit(q.limit))
        .await?;
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
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ProposeRequest>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let p = state
        .service()
        .propose(
            m,
            req.opportunity_id,
            &req.title,
            &req.body,
            req.price_cents,
            req.currency,
            req.kb_ref_ids,
        )
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(p)))
}

async fn list_proposals(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let m = auth(&headers)?;
    let v = state
        .service()
        .list_proposals(m, clamp_limit(q.limit))
        .await?;
    Ok(Json(json!({"proposals":v})))
}

/// Resolve the calling member from the bearer token.
///
/// A missing, malformed, wrongly-signed or non-UUID-subject token is a flat
/// `401`; the service never falls back to a default member and never reads a
/// member id from the request.
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

/// Parse a contract `OpportunityStatus` token.
///
/// Returns `None` only for the empty string, which the list filter uses to mean
/// "no status filter"; the transition endpoint requires a real status and
/// rejects an empty one.
fn parse_status(s: &str) -> Result<Option<OpportunityStatus>, Error> {
    if s.is_empty() {
        return Ok(None);
    }
    OpportunityStatus::parse(s)
        .map(Some)
        .ok_or_else(|| Error::Validation(format!("unknown status {s}")))
}

fn parse_source(s: &str) -> Result<Source, Error> {
    Source::parse(s).ok_or_else(|| Error::Validation(format!("unknown source {s}")))
}
