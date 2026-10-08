//! Orchestrator HTTP handlers — canonical /api/v1/briefing/* namespace.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use lcc_auth::Caller;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::Briefing;
use crate::error::Error;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BriefingQuery {
    pub date: Option<NaiveDate>,
}

/// Resolves the acting member for a member-scoped route.
///
/// The authenticated identity comes from the token (injected by the
/// `require_permission` route layer), never from the path. A path `:member_id`
/// that disagrees with the verified subject is refused with **404** rather than
/// 403: a 403 would confirm that the member exists, which is exactly the
/// existence oracle the security design forbids. 404 is also what a genuinely
/// unknown member would produce, so the two are indistinguishable.
fn require_own_member(caller: &Caller, path_member_id: Uuid) -> Result<Uuid, Error> {
    if caller.member_id == path_member_id {
        Ok(caller.member_id)
    } else {
        Err(Error::NotFound(format!("member {path_member_id}")))
    }
}

/// GET /api/v1/members/{memberId}/briefing/today
pub async fn briefing_today(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Path(member_id): Path<Uuid>,
    Query(q): Query<BriefingQuery>,
) -> Result<Json<Briefing>, Error> {
    let member_id = require_own_member(&caller, member_id)?;
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
    Extension(caller): Extension<Caller>,
    Path(member_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let member_id = require_own_member(&caller, member_id)?;
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
