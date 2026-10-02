//! Compliance Governor — single arbiter for permitted external actions.
//!
//! Implements the 8 sequential guards from Source Technical Design Spec §11
//! (Backend Design Concept §27). p99 evaluation SLO < 200ms (Non-Negotiable).
//!
//! ### Boundary
//! - This is the only service that issues `permit_token`s.
//! - Only the Integration Gateway verifies `permit_token`s (Source Non-Negotiable §2).
//! - Guard evaluation is **synchronous inline** within the request path
//!   (Non-Negotiable §12) — no fire-and-forget variant.

pub mod api;
pub mod config;
pub mod error;
pub mod guards;
pub mod health;
pub mod http;
pub mod scoring;
pub mod sign;
pub mod state;

use chrono::{DateTime, Utc};
use lcc_audit_client::AuditOutcome;
use lcc_compliance::config::ComplianceConfig;
use lcc_compliance::permit_token::PermitSigner;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// GovernorDeps — everything a guard evaluation needs. Passed by reference
/// to each guard via the `Guard` trait.
#[derive(Clone)]
pub struct GovernorDeps {
    pub config: Arc<ComplianceConfig>,
    pub redis: deadpool_redis::Pool,
    pub db: sqlx::PgPool,
    pub permit_issuer: Arc<PermitSigner>,
    pub audit: lcc_audit_client::AuditClient,
    pub scoring_client: Arc<scoring::ScoringClient>,
}

/// GovernorDecision — outcome of evaluating a candidate action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GovernorDecision {
    Permit(PermitMetadata),
    Deny { failed_guard: String, reason: String },
    Defer { reason: String, retry_after_ms: u64 },
}

/// PermitMetadata — what the Governor returns on PERMIT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermitMetadata {
    pub permit_token: String,
    pub approval_id: String,
    pub delay_ms: u64,
    pub reserve_consumed: bool,
    pub active_compliance_config_version: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// CandidateAction — input to evaluate_action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateAction {
    pub id: Uuid,
    pub action_type: lcc_compliance::action::ActionType,
    pub member_id: Uuid,
    pub risk_tier: lcc_compliance::action::RiskTier,
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

/// AccountState — input to evaluate_action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountState {
    pub member_id: Uuid,
    pub h_c: f64,
    pub h_c_computed_at: DateTime<Utc>,
    pub is_restricted: bool,
    pub restricted_since: Option<DateTime<Utc>>,
    pub restricted_reason: Option<String>,
    pub warmup_days_remaining: u32,
    pub days_active: u32,
    pub active_compliance_config_version: String,
}

impl AccountState {
    /// Placeholder account state for tests / local dev. Member_id is
    /// nil so callers cannot accidentally use this in a real RPC.
    pub fn placeholder() -> Self {
        Self {
            member_id: Uuid::nil(),
            h_c: 0.65,
            h_c_computed_at: Utc::now(),
            is_restricted: false,
            restricted_since: None,
            restricted_reason: None,
            warmup_days_remaining: 0,
            days_active: 30,
            active_compliance_config_version: "test".into(),
        }
    }
}

/// EvaluateResult — full output of evaluate_action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluateResult {
    pub decision: GovernorDecision,
    pub guard_evaluation_duration_ms: u64,
}

/// Evaluate a candidate action against all 8 guards. Sequential; ALL must pass.
pub async fn evaluate_action(
    action: &CandidateAction,
    account: &AccountState,
    deps: &GovernorDeps,
) -> EvaluateResult {
    let start = std::time::Instant::now();

    // The 8 guards in source-spec order. Partial-pass = deny (axiom 3).
    let guards = guards::build_guard_stack(deps);
    for guard in guards.iter() {
        let result = guard.check(action, account, deps).await;
        if !result.passed {
            let failed_guard = guard.name().to_string();
            let reason = result.reason;
            deps.audit
                .record_quick(lcc_audit_client::AuditEvent::new(
                    format!("system:compliance-governor"),
                    "guard.deny",
                    "candidate_action",
                )
                .resource_id(action.id)
                .member_id(action.member_id)
                .outcome(AuditOutcome::Denied)
                .reason(format!("{failed_guard}: {reason}"))
                .idempotency_key(action.idempotency_key.clone()));

            return EvaluateResult {
                decision: GovernorDecision::Deny { failed_guard, reason },
                guard_evaluation_duration_ms: start.elapsed().as_millis() as u64,
            };
        }
    }

    // All guards passed — issue permit.
    let approval_id = Uuid::now_v7();
    let member_id_str = action.member_id.to_string();
    let action_id_str = action.id.to_string();
    let claims = lcc_compliance::permit_token::PermitClaimsBuilder::new(
        &member_id_str,
        &action_id_str,
        action.action_type.as_str(),
        action.risk_tier.as_str(),
        &account.active_compliance_config_version,
    )
    .approval_id(Some(approval_id.to_string()))
    .ttl_seconds(deps.config.permit_token_ttl_seconds as i64)
    .build();

    let permit_token = match deps.permit_issuer.issue(&claims) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "failed to issue permit_token");
            return EvaluateResult {
                decision: GovernorDecision::Deny {
                    failed_guard: "permit_token_issuance".into(),
                    reason: e.to_string(),
                },
                guard_evaluation_duration_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let permit = PermitMetadata {
        permit_token,
        approval_id: approval_id.to_string(),
        delay_ms: 0,
        reserve_consumed: false,
        active_compliance_config_version: account.active_compliance_config_version.clone(),
        issued_at: chrono::DateTime::from_timestamp(claims.iat, 0).unwrap_or_else(|| Utc::now()),
        expires_at: chrono::DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(|| Utc::now()),
    };

    deps.audit
        .record_quick(lcc_audit_client::AuditEvent::new(
            "system:compliance-governor",
            "guard.permit",
            "candidate_action",
        )
        .resource_id(action.id)
        .member_id(action.member_id)
        .outcome(AuditOutcome::Success)
        .idempotency_key(action.idempotency_key.clone()));

    EvaluateResult {
        decision: GovernorDecision::Permit(permit),
        guard_evaluation_duration_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lcc_compliance::action::{ActionType, RiskTier};

    fn test_action() -> CandidateAction {
        CandidateAction {
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

    fn test_account() -> AccountState {
        AccountState {
            member_id: Uuid::new_v4(),
            h_c: 0.8,
            h_c_computed_at: Utc::now(),
            is_restricted: false,
            restricted_since: None,
            restricted_reason: None,
            warmup_days_remaining: 0,
            days_active: 100,
            active_compliance_config_version: "ccfg-test".into(),
        }
    }

    #[test]
    fn candidate_serializes() {
        let a = test_action();
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("like"));
    }

    #[test]
    fn account_serializes() {
        let a = test_account();
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("0.8"));
    }
}
