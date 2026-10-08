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
    Extension, Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::admin::ActivateConfigVersionRequest;
use crate::api::admin::ActivateConfigVersionResponse;
use crate::api::{
    activate_config, evaluate_action_http, list_config_versions, propose_config, review_config,
    ProposeConfigVersionRequest, ProposeConfigVersionResponse, ReviewConfigVersionRequest,
    ReviewConfigVersionResponse,
};
use crate::error::GovernorError;
use crate::state::GovernorDeps;

pub mod admin_rest;

/// Builds the governor router.
///
/// `jwt_verifier` is threaded in rather than read from the environment so the
/// admin subtree's authorization gate is testable with an explicit secret and
/// so the decoding key is parsed once at startup instead of per request.
pub fn build_router(
    deps: Arc<GovernorDeps>,
    metrics: Arc<lcc_observability::metrics::Metrics>,
    jwt_verifier: lcc_auth::SharedVerifier,
) -> Router {
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
        // ---------------------------------------------------------------------
        // F-AUDIT-60: every route above is a state change on the compliance
        // configuration that decides what the platform is permitted to send —
        // the highest-consequence surface in the system — and none of them
        // performed any service-level check. The gateway verifies a token
        // before proxying, but a caller able to reach this service directly
        // (cluster-internal path, misconfigured ingress, an SSRF elsewhere)
        // could propose, review and activate a config version with no
        // credential at all.
        //
        // The gate is applied as a `route_layer` over the admin subtree rather
        // than inside each handler, because per-handler checks are precisely
        // what produced the original gap: the next handler added under this
        // path would be unprotected unless someone remembered. With the layer
        // here, a new route is protected by default and cannot be reached by
        // an alternative execution path that skips the handler body.
        //
        // `ManageComplianceConfig` is the permission from
        // `lcc_auth::rbac`; only Owner and Admin hold it. Authenticated
        // callers without it get 403, missing/invalid credentials get 401.
        // ---------------------------------------------------------------------
        .route_layer(axum::middleware::from_fn(move |request, next| {
            let v = jwt_verifier.clone();
            async move {
                lcc_auth::guard::require_permission(
                    v,
                    lcc_auth::Permission::ManageComplianceConfig,
                    request,
                    next,
                )
                .await
            }
        }))
        // Legacy internal-only routes kept for backward compatibility with any
        // internal callers during the migration window. These are deprecated and
        // will be removed once the gateway is the sole caller. They are
        // registered here, before `.with_state(..)` erases `Arc<GovernorDeps>`,
        // because their handlers take that state directly.
        .route("/internal/governor/evaluate", post(evaluate_action_http))
        .route("/internal/compliance/config/propose", post(propose_config))
        .route("/internal/compliance/config/review", post(review_config))
        .route("/internal/compliance/config/activate", post(activate_config))
        .route("/internal/compliance/config/list", get(list_config_versions))
        // Metrics rides as an `Extension` so it does not have to share the
        // router's `Arc<GovernorDeps>` state type.
        .layer(Extension(metrics))
        .with_state(deps)
}

// ----- canonical REST adapter wrappers -----

async fn propose_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<ProposeConfigVersionRequest>,
) -> Result<Json<ProposeConfigVersionResponse>, GovernorError> {
    let (_status, resp) = propose_config(State(deps), Json(req)).await?;
    Ok(resp)
}

async fn list_config_versions_handler(
    State(deps): State<Arc<GovernorDeps>>,
) -> Result<Json<admin_rest::ListConfigVersionsRestResponse>, GovernorError> {
    let (_status, resp) = list_config_versions(State(deps)).await?;
    // Re-shape the internal DTO into the canonical REST shape.
    Ok(Json(admin_rest::ListConfigVersionsRestResponse {
        items: resp
            .0
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
        active_version: resp.0.active_version,
    }))
}

async fn review_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Path(version_id): Path<Uuid>,
    Json(req): Json<admin_rest::ReviewConfigVersionRestRequest>,
) -> Result<Json<ReviewConfigVersionResponse>, GovernorError> {
    // `version_id` is addressed by the route path, not the body, so the two
    // sources are combined here rather than written back onto the body DTO.
    let internal = ReviewConfigVersionRequest {
        version_id,
        reviewer_user_id: req.reviewer_user_id,
        reviewer_notes: req.notes.unwrap_or_default(),
        approve: req.approve,
    };
    let (_status, resp) = review_config(State(deps), Json(internal)).await?;
    Ok(resp)
}

async fn activate_config_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Path(version_id): Path<Uuid>,
    Json(req): Json<admin_rest::ActivateConfigVersionRestRequest>,
) -> Result<Json<ActivateConfigVersionResponse>, GovernorError> {
    let internal = ActivateConfigVersionRequest {
        version_id,
        reviewer_a_user_id: req.reviewer_a_user_id,
        reviewer_a_signature: req.reviewer_a_signature,
        reviewer_b_user_id: req.reviewer_b_user_id,
        reviewer_b_signature: req.reviewer_b_signature,
    };
    let (_status, resp) = activate_config(State(deps), Json(internal)).await?;
    Ok(resp)
}

async fn evaluate_action_http_handler(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<crate::api::evaluate::EvaluateActionRequestDto>,
) -> Result<Json<crate::api::evaluate::EvaluateActionResponseDto>, crate::error::GovernorError> {
    let (_status, resp) = evaluate_action_http(State(deps), Json(req)).await?;
    Ok(resp)
}
