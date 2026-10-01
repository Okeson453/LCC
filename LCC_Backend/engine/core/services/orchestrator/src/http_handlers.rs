//! Orchestrator HTTP handlers — canonical /api/v1/briefing/* namespace.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Briefing;
use crate::error::Error;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BriefingQuery {
    pub date: Option<NaiveDate>,
}

/// GET /api/v1/members/{memberId}/briefing/today
pub async fn briefing_today(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
    Query(q): Query<BriefingQuery>,
    _headers: HeaderMap,
) -> Result<Json<Briefing>, Error> {
    let date = q.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let briefing = state.service().assemble_briefing(member_id, date).await?;
    Ok(Json(briefing))
}

#[derive(Debug, Serialize)]
pub struct ScheduleAck {
    pub accepted: bool,
    pub trace_id: Uuid,
}

/// POST /api/v1/members/{memberId}/briefing/refresh
/// Triggers an on-demand briefing refresh (e.g., from the dashboard's
/// "regenerate" button). Returns 202 with an acknowledgment; the actual
/// briefing payload is fetched via the GET endpoint above.
pub async fn briefing_refresh(
    State(state): State<AppState>,
    Path(member_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    // Synchronously assemble and emit the refresh event so the
    // realtime-svc picks it up immediately.
    let _ = state
        .service()
        .assemble_briefing(member_id, chrono::Utc::now().date_naive())
        .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(ScheduleAck {
            accepted: true,
            trace_id: Uuid::new_v4(),
        }),
    ))
}

use axum::extract::Query;
