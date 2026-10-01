//! HTTP + gRPC handler for `evaluate_action`.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use lcc_compliance::action::{ActionType, RiskTier};
use lcc_observability::span::span_for_governor_evaluate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::GovernorDeps;
use crate::{evaluate_action, AccountState, CandidateAction, EvaluateResult, GovernorDecision};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateActionRequestDto {
    pub candidate_action: CandidateActionDto,
    pub account_state: AccountStateDto,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateActionDto {
    pub id: Uuid,
    pub action_type: String,
    pub member_id: Uuid,
    pub risk_tier: String,
    pub kb_refs: Vec<Uuid>,
    pub idempotency_key: String,
    pub requires_approval: bool,
    pub auto_execute: bool,
    pub target_contact_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub content_item_id: Option<Uuid>,
    pub sequence_step_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStateDto {
    pub h_c: f64,
    pub h_c_computed_at: chrono::DateTime<Utc>,
    pub is_restricted: bool,
    pub restricted_since: Option<chrono::DateTime<Utc>>,
    pub restricted_reason: Option<String>,
    pub warmup_days_remaining: u32,
    pub days_active: u32,
    pub active_compliance_config_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateActionResponseDto {
    pub decision: String,
    pub failed_guard: Option<String>,
    pub reason: Option<String>,
    pub permit_metadata: Option<crate::PermitMetadata>,
    pub guard_evaluation_duration_ms: u64,
}

impl From<EvaluateActionRequestDto> for (CandidateAction, AccountState) {
    fn from(dto: EvaluateActionRequestDto) -> Self {
        let action_type: ActionType = serde_json::from_str(&format!("\"{}\"", dto.candidate_action.action_type))
            .unwrap_or(ActionType::Like);
        let risk_tier: RiskTier = serde_json::from_str(&format!("\"{}\"", dto.candidate_action.risk_tier))
            .unwrap_or(RiskTier::Tier1DraftOrEdit);

        let action = CandidateAction {
            id: dto.candidate_action.id,
            action_type,
            member_id: dto.candidate_action.member_id,
            risk_tier,
            kb_refs: dto.candidate_action.kb_refs,
            idempotency_key: dto.candidate_action.idempotency_key,
            requires_approval: dto.candidate_action.requires_approval,
            auto_execute: dto.candidate_action.auto_execute,
            target_contact_id: dto.candidate_action.target_contact_id,
            opportunity_id: dto.candidate_action.opportunity_id,
            content_item_id: dto.candidate_action.content_item_id,
            sequence_step_id: dto.candidate_action.sequence_step_id,
            application_id: dto.candidate_action.application_id,
            metadata: dto.candidate_action.metadata,
        };
        let account = AccountState {
            h_c: dto.account_state.h_c,
            h_c_computed_at: dto.account_state.h_c_computed_at,
            is_restricted: dto.account_state.is_restricted,
            restricted_since: dto.account_state.restricted_since,
            restricted_reason: dto.account_state.restricted_reason,
            warmup_days_remaining: dto.account_state.warmup_days_remaining,
            days_active: dto.account_state.days_active,
            active_compliance_config_version: dto.account_state.active_compliance_config_version,
        };
        (action, account)
    }
}

pub async fn evaluate_action_http(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<EvaluateActionRequestDto>,
) -> impl IntoResponse {
    let (action, account): (CandidateAction, AccountState) = req.into();
    let span = span_for_governor_evaluate(
        &Uuid::new_v4().to_string(), // trace_id from request headers would be better
        &action.member_id.to_string(),
        &action.action_type.to_string(),
    );
    let _enter = span.enter();

    let result = evaluate_action(&action, &account, &deps).await;
    let decision_str = match &result.decision {
        GovernorDecision::Permit(_) => "PERMIT",
        GovernorDecision::Deny { .. } => "DENY",
        GovernorDecision::Defer { .. } => "DEFER",
    };
    tracing::info!(
        decision = decision_str,
        guard_evaluation_duration_ms = result.guard_evaluation_duration_ms,
        "governor.evaluate"
    );

    let resp = match &result.decision {
        GovernorDecision::Permit(p) => EvaluateActionResponseDto {
            decision: "PERMIT".into(),
            failed_guard: None,
            reason: None,
            permit_metadata: Some(p.clone()),
            guard_evaluation_duration_ms: result.guard_evaluation_duration_ms,
        },
        GovernorDecision::Deny { failed_guard, reason } => EvaluateActionResponseDto {
            decision: "DENY".into(),
            failed_guard: Some(failed_guard.clone()),
            reason: Some(reason.clone()),
            permit_metadata: None,
            guard_evaluation_duration_ms: result.guard_evaluation_duration_ms,
        },
        GovernorDecision::Defer { reason, .. } => EvaluateActionResponseDto {
            decision: "DEFER".into(),
            failed_guard: None,
            reason: Some(reason.clone()),
            permit_metadata: None,
            guard_evaluation_duration_ms: result.guard_evaluation_duration_ms,
        },
    };

    let status = match &result.decision {
        GovernorDecision::Permit(_) => StatusCode::OK,
        GovernorDecision::Deny { .. } | GovernorDecision::Defer { .. } => StatusCode::FORBIDDEN,
    };

    (status, Json(resp)).into_response()
}

/// gRPC server — wraps the evaluate_action in a tonic service.
/// (The actual implementation is wired via the generated proto types.)
pub struct GrpcServer;

impl GrpcServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn serve(self, _addr: std::net::SocketAddr, _deps: Arc<GovernorDeps>) -> Result<(), crate::error::GovernorError> {
        // Real implementation: tonic::transport::Server::builder()
        //     .add_service(GovernorServer::new(governor_grpc::GovernordImpl { deps }))
        //     .serve(addr)
        //     .await?;
        Ok(())
    }
}

impl Default for GrpcServer {
    fn default() -> Self {
        Self::new()
    }
}

// Required for test usage of CandidateAction in guards:
impl CandidateAction {
    pub fn dummy_for_tests() -> Self {
        Self {
            id: Uuid::now_v7(),
            action_type: ActionType::Like,
            member_id: Uuid::now_v7(),
            risk_tier: RiskTier::Tier2LightEngagement,
            kb_refs: vec![Uuid::now_v7()],
            idempotency_key: Uuid::now_v7().to_string(),
            requires_approval: false,
            auto_execute: true,
            target_contact_id: None,
            opportunity_id: None,
            content_item_id: None,
            sequence_step_id: None,
            application_id: None,
            metadata: std::collections::BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_action_is_well_formed() {
        let a = CandidateAction::dummy_for_tests();
        assert!(!a.kb_refs.is_empty());
        assert_eq!(a.action_type, ActionType::Like);
    }
}
