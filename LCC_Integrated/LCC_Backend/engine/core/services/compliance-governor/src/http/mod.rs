//! Compliance Governor HTTP router (canonical namespace).
//!
//! Routes:
//!   GET    /healthz
//!   GET    /readyz
//!   POST   /api/v1/admin/compliance/config-versions         propose_config
//!   GET    /api/v1/admin/compliance/config-versions         list_config_versions
//!   POST   /api/v1/admin/compliance/config-versions/{id}/activate
//!   POST   /api/v1/admin/compliance/config-versions/{id}/review
//!   POST   /api/v1/admin/evaluate                           evaluate_action_http

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::{
    activate_config, evaluate_action_http, list_config_versions, propose_config, review_config,
    ProposeConfigVersionRequest, ProposeConfigVersionResponse, ReviewConfigVersionRequest,
    ReviewConfigVersionResponse,
};
use crate::api::admin::ActivateConfigVersionRequest;
use crate::api::admin::ActivateConfigVersionResponse;
use crate::error::GovernorError;
use crate::state::GovernorDeps;

pub mod admin_rest;

pub fn build_router(deps: Arc<GovernorDeps>) -> Router {
    Router::new()
        .route("/healthz", get(crate::health::healthz))
        .route("/readyz", get(crate::health::readyz))
        .route("/metrics", get(crate::health::metrics_handler))
        // Canonical namespace.
        .route(
            "/api/v1/admin/compliance/config-versions",
            post(propose_config_handler).get(list_config_versions_handler),
        )
        .route(
            "/api/v1/admin/compliance/config-versions/:version_id/review",
            post(review_config_handler),
        )
        .route(
            "/api/v1/admin/compliance/config-versions/:version_id/activate",
            post(activate_config_handler),
        )
        .route("/api/v1/admin/evaluate", post(evaluate_action_http_handler))
        .with_state(deps)
}

// ----- canonical REST adapter wrappers -----

async fn propose_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<ProposeConfigVersionRequest>,
) -> Result<Json<ProposeConfigVersionResponse>, GovernorError> {
    let resp = propose_config(State(deps), Json(req)).await?;
    Ok(Json(resp.1))
}

async fn list_config_versions_handler(
    State(deps): State<Arc<GovernorDeps>>,
) -> Result<Json<admin_rest::ListConfigVersionsRestResponse>, GovernorError> {
    let resp = list_config_versions(State(deps)).await?;
    // Re-shape the internal DTO into the canonical REST shape.
    Ok(Json(admin_rest::ListConfigVersionsRestResponse {
        items: resp
            .1
            .versions
            .into_iter()
            .map(|v| admin_rest::ComplianceConfigVersionDto {
                id: v.version_id,
                version: v.version,
                status: if v.active { "active" } else { "draft" }.to_string(),
                two_reviewer_signed_by: v.signed_by,
                activated_at: v.activated_at,
                previous_id: None,
            })
            .collect(),
        active_version: resp.1.active_version,
    }))
}

async fn review_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Path(version_id): Path<Uuid>,
    Json(mut req): Json<admin_rest::ReviewConfigVersionRestRequest>,
) -> Result<Json<ReviewConfigVersionResponse>, GovernorError> {
    req.version_id = version_id;
    let internal = ReviewConfigVersionRequest {
        version_id: req.version_id,
        reviewer_user_id: req.reviewer_user_id,
        reviewer_notes: req.notes.unwrap_or_default(),
        approve: req.approve,
    };
    let resp = review_config(State(deps), Json(internal)).await?;
    Ok(Json(resp.1))
}

async fn activate_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Path(version_id): Path<Uuid>,
    Json(mut req): Json<admin_rest::ActivateConfigVersionRestRequest>,
) -> Result<Json<ActivateConfigVersionResponse>, GovernorError> {
    req.version_id = version_id;
    let internal = ActivateConfigVersionRequest {
        version_id: req.version_id,
        reviewer_a_user_id: req.reviewer_a_user_id,
        reviewer_a_signature: req.reviewer_a_signature,
        reviewer_b_user_id: req.reviewer_b_user_id,
        reviewer_b_signature: req.reviewer_b_signature,
    };
    let resp = activate_config(State(deps), Json(internal)).await?;
    Ok(Json(resp.1))
}

async fn evaluate_action_http_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<crate::api::evaluate::EvaluateActionRequestDto>,
) -> Result<
    Json<crate::api::evaluate::EvaluateActionResponseDto>,
    crate::error::GovernorError,
> {
    let resp = evaluate_action_http(State(deps), Json(req)).await?;
    Ok(Json(resp.1))
}
