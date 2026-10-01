//! identity-svc HTTP handlers.
//!
//! Implements the canonical `/api/v1/...` namespace per
//! `contract_audit/openapi/lcc-api-canonical.yaml`.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    GoalMode, LinkedInStart, Member, MemberSettings, MemberSettingsUpdate, TokenPair,
};
use crate::error::Error;
use crate::service::JwtClaims;
use crate::state::AppState;

// ============================ AUTH ============================

#[derive(Debug, Deserialize)]
pub struct LinkedInStartQuery {
    pub return_to: Option<String>,
}

/// GET /api/v1/auth/linkedin/start
pub async fn auth_linkedin_start(
    State(state): State<AppState>,
    Query(q): Query<LinkedInStartQuery>,
) -> Result<Json<LinkedInStart>, Error> {
    let start = state.service().begin_oauth(q.return_to)?;
    Ok(Json(start))
}

#[derive(Debug, Deserialize)]
pub struct LinkedInCallbackQuery {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub code_verifier: Option<String>,
}

/// GET /api/v1/auth/linkedin/callback
pub async fn auth_linkedin_callback(
    State(state): State<AppState>,
    Query(q): Query<LinkedInCallbackQuery>,
) -> Result<Json<AuthCallbackResponse>, Error> {
    let verifier = q.code_verifier.ok_or_else(|| {
        Error::BadRequest(
            "missing code_verifier (PKCE); the frontend must hold the value returned from /start"
                .into(),
        )
    })?;
    let (member, tokens) = state
        .service()
        .complete_oauth(&q.code, &q.state, &verifier)
        .await?;
    Ok(Json(AuthCallbackResponse {
        member_id: member.id,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        role: member.role.as_str().to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct AuthCallbackResponse {
    pub member_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// POST /api/v1/auth/refresh
pub async fn auth_refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenPair>, Error> {
    let pair = state.service().refresh(&req.refresh_token).await?;
    Ok(Json(pair))
}

/// POST /api/v1/auth/logout
pub async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, Error> {
    let claims = require_member(&state, &headers)?;
    state.service().logout(claims.sub_as_uuid()?).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================ MEMBERS ============================

/// GET /api/v1/members/me
pub async fn members_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Member>, Error> {
    let claims = require_member(&state, &headers)?;
    let m = state.service().get_member(claims.sub_as_uuid()?).await?;
    Ok(Json(m))
}

/// PATCH /api/v1/members/me/settings
pub async fn members_me_settings_patch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(update): Json<MemberSettingsUpdate>,
) -> Result<Json<Member>, Error> {
    let claims = require_member(&state, &headers)?;
    let current = state.service().get_member(claims.sub_as_uuid()?).await?;
    let updated = state
        .service()
        .update_settings(claims.sub_as_uuid()?, update, current.version)
        .await?;
    Ok(Json(updated))
}

/// GET /api/v1/members/me/settings
pub async fn members_me_settings_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MemberSettings>, Error> {
    let claims = require_member(&state, &headers)?;
    let s = state.service().get_settings(claims.sub_as_uuid()?).await?;
    Ok(Json(s))
}

/// GET /api/v1/members/{memberId}
/// Requires role = Admin or Auditor to view a non-self member.
pub async fn members_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(member_id): Path<Uuid>,
) -> Result<Json<Member>, Error> {
    let claims = require_member(&state, &headers)?;
    if claims.sub_as_uuid()? != member_id && !claims.role_is("admin") && !claims.role_is("auditor") {
        return Err(Error::Forbidden(
            "non-self reads require role=admin or auditor".into(),
        ));
    }
    let m = state.service().get_member(member_id).await?;
    Ok(Json(m))
}

// ============================ Helpers ============================

fn require_member(state: &AppState, headers: &HeaderMap) -> Result<JwtClaims, Error> {
    let token = extract_bearer(headers)?;
    state.service().verify_access_token(&token)
}

fn extract_bearer(headers: &HeaderMap) -> Result<String, Error> {
    let raw = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| Error::Unauthorized("missing Authorization header".into()))?
        .to_str()
        .map_err(|_| Error::Unauthorized("invalid Authorization header".into()))?;
    let token = raw
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::Unauthorized("Authorization must be Bearer <jwt>".into()))?
        .trim();
    if token.is_empty() {
        return Err(Error::Unauthorized("empty bearer token".into()));
    }
    Ok(token.to_string())
}

// ---------- Claim helpers ----------

impl JwtClaims {
    pub fn sub_as_uuid(&self) -> Result<Uuid, Error> {
        Uuid::parse_str(&self.sub).map_err(|_| Error::Unauthorized("invalid sub".into()))
    }

    pub fn role_is(&self, role: &str) -> bool {
        self.role.eq_ignore_ascii_case(role)
    }
}

// We need GoalMode FromStr for any path that may convert; keep here for clarity.
impl GoalMode {
    #[allow(dead_code)]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "job_hunting" => Some(Self::JobHunting),
            "client_acquisition" => Some(Self::ClientAcquisition),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }
}
