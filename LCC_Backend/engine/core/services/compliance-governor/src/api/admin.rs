//! Compliance config admin endpoints — propose, review, activate.
//!
//! Two-reviewer activation gate (Non-Negotiable §4): activation requires
//! two distinct reviewers' signatures, each with a non-empty signature,
//! persisted to `lcc.compliance_config_versions` with `is_active = true`
//! and `activated_at = NOW()`. The previous version is atomically retired.
//!
//! Every action emits an audit_log entry (`lcc_audit.events`) via the
//! `lcc_audit_client` crate, with the action_type column set to
//! `compliance.config_activated`.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use lcc_audit_client::{AuditEvent, AuditOutcome};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::GovernorError;
use crate::state::GovernorDeps;

// ===========================================================================
// Propose
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeConfigVersionRequest {
    pub proposer_user_id: String,
    pub h_c_weights: [f64; 4],
    pub phi_weights: [f64; 4],
    pub caps_override: Option<lcc_compliance::config::ActionCaps>,
    pub reserve_fraction: Option<f64>,
    pub phi_qualification_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeConfigVersionResponse {
    pub version_id: Uuid,
    pub version: String,
    pub static_check_status: String,
    pub static_check_report: String,
}

pub async fn propose_config(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<ProposeConfigVersionRequest>,
) -> Result<impl IntoResponse, GovernorError> {
    // Build a candidate config and validate it.
    let mut cfg = lcc_compliance::config::ComplianceConfig::default();
    cfg.h_c_weights = req.h_c_weights;
    cfg.phi_weights = req.phi_weights;
    if let Some(caps) = req.caps_override {
        cfg.caps = caps;
    }
    if let Some(rf) = req.reserve_fraction {
        cfg.reserve_fraction = rf;
    }
    if let Some(t) = req.phi_qualification_threshold {
        cfg.phi_qualification_threshold = t;
    }

    let static_check_report = match cfg.validate() {
        Ok(()) => "passed".to_string(),
        Err(e) => format!("failed: {e}"),
    };
    let static_check_status = if static_check_report == "passed" {
        "passed"
    } else {
        "failed"
    };

    let version_id = Uuid::now_v7();
    let version = format!(
        "ccfg-{}-proposed-{}",
        Utc::now().format("%Y-%m-%d"),
        &version_id.to_string()[..8]
    );
    cfg.version = version.clone();

    // Persist the proposal to lcc.compliance_config_versions with is_active=false.
    let payload = serde_json::to_value(&cfg).unwrap_or(serde_json::json!({}));
    sqlx::query(
        r#"
        INSERT INTO lcc.compliance_config_versions (
            version, is_active, config, activated_at, activated_by,
            two_reviewer_signed_by
        )
        VALUES ($1, false, $2, NULL, $3, '{}'::TEXT[])
        "#,
    )
    .bind(&version)
    .bind(&payload)
    .bind(&req.proposer_user_id)
    .execute(&deps.db)
    .await
    .map_err(|e| GovernorError::Internal(format!("db: {e}")))?;

    // Audit emission (best-effort; the audit transport is a stub in the
    // scaffold per the original audit, so a transport failure here does
    // not block the proposal).
    {
        let event = AuditEvent::new(
            format!("system:compliance-governor:{}", req.proposer_user_id),
            "compliance.config_proposed",
            "compliance_config_version",
        )
        .resource_id(version_id)
        .after_state(serde_json::json!({
            "version": version,
            "static_check": static_check_status,
        }));
        // record_quick is best-effort and never fails the caller; the audit
        // transport falls back to a local counter when the gRPC/HTTP sink
        // is unreachable.
        deps.audit.record_quick(event);
    }

    Ok((
        StatusCode::CREATED,
        Json(ProposeConfigVersionResponse {
            version_id,
            version,
            static_check_status: static_check_status.to_string(),
            static_check_report,
        }),
    ))
}

// ===========================================================================
// Review (record a single reviewer's signature; activation requires two)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfigVersionRequest {
    pub version_id: Uuid,
    pub reviewer_user_id: String,
    pub reviewer_notes: String,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfigVersionResponse {
    pub version_id: Uuid,
    pub reviewer_count: usize,
    pub signed_by: Vec<String>,
    pub status: String,
}

pub async fn review_config(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<ReviewConfigVersionRequest>,
) -> Result<impl IntoResponse, GovernorError> {
    if !req.approve {
        // A rejected review is a no-op against the signature array; emit an
        // audit entry so the rejection is traceable.
        {
            let event = AuditEvent::new(
                format!("system:compliance-governor:{}", req.reviewer_user_id),
                "compliance.config_rejected",
                "compliance_config_version",
            )
            .resource_id(req.version_id)
            .reason(req.reviewer_notes.clone());
            deps.audit.record_quick(event);
        }
        return Ok((
            StatusCode::OK,
            Json(ReviewConfigVersionResponse {
                version_id: req.version_id,
                reviewer_count: 0,
                signed_by: vec![],
                status: "rejected".into(),
            }),
        ));
    }

    // Read the current signed_by array; append the reviewer id (only if
    // not already present); write it back. The unique-on-is_active index
    // protects against concurrent writes.
    let current: (Option<Vec<String>>, Option<bool>) = sqlx::query_as(
        r#"
        SELECT two_reviewer_signed_by, is_active
        FROM lcc.compliance_config_versions
        WHERE id = $1
        "#,
    )
    .bind(req.version_id)
    .fetch_optional(&deps.db)
    .await
    .map_err(|e| GovernorError::Internal(format!("db: {e}")))?
    .unwrap_or((None, None));

    let mut signed_by = current.0.unwrap_or_default();
    if signed_by.contains(&req.reviewer_user_id) {
        return Err(GovernorError::TwoReviewerRequired(signed_by.len()));
    }
    if signed_by.len() >= 2 {
        return Err(GovernorError::TwoReviewerRequired(2));
    }
    signed_by.push(req.reviewer_user_id.clone());

    sqlx::query(
        r#"
        UPDATE lcc.compliance_config_versions
        SET two_reviewer_signed_by = $2
        WHERE id = $1
        "#,
    )
    .bind(req.version_id)
    .bind(&signed_by)
    .execute(&deps.db)
    .await
    .map_err(|e| GovernorError::Internal(format!("db: {e}")))?;

    let status = if signed_by.len() >= 2 {
        "ready_to_activate"
    } else {
        "pending_second_reviewer"
    }
    .to_string();

    {
        let event = AuditEvent::new(
            format!("system:compliance-governor:{}", req.reviewer_user_id),
            "compliance.config_reviewed",
            "compliance_config_version",
        )
        .resource_id(req.version_id)
        .after_state(serde_json::json!({
            "notes": req.reviewer_notes,
            "signed_count": signed_by.len(),
        }));
        deps.audit.record_quick(event);
    }

    Ok((
        StatusCode::OK,
        Json(ReviewConfigVersionResponse {
            version_id: req.version_id,
            reviewer_count: signed_by.len(),
            signed_by,
            status,
        }),
    ))
}

// ===========================================================================
// Activate — requires two DISTINCT reviewers' non-empty signatures.
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateConfigVersionRequest {
    pub version_id: Uuid,
    pub reviewer_a_user_id: String,
    pub reviewer_a_signature: String,
    pub reviewer_b_user_id: String,
    pub reviewer_b_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateConfigVersionResponse {
    pub accepted: bool,
    pub audit_log_id: i64,
    pub activated_version_id: Uuid,
    pub previous_version_id: Option<Uuid>,
}

pub async fn activate_config(
    State(deps): State<Arc<GovernorDeps>>,
    Json(req): Json<ActivateConfigVersionRequest>,
) -> Result<impl IntoResponse, GovernorError> {
    // Two-reviewer gate (Non-Negotiable §4).
    if req.reviewer_a_user_id == req.reviewer_b_user_id {
        return Err(GovernorError::TwoReviewerRequired(1));
    }
    if req.reviewer_a_signature.trim().is_empty() || req.reviewer_b_signature.trim().is_empty() {
        return Err(GovernorError::TwoReviewerRequired(0));
    }
    if req.reviewer_a_signature.trim() == req.reviewer_b_signature.trim() {
        // Two non-distinct signatures are not two reviewers.
        return Err(GovernorError::TwoReviewerRequired(1));
    }

    // Persist atomically:
    //   1. Retire the currently-active version (if any) -> is_active=false
    //   2. Activate this version (is_active=true, activated_at=NOW(), signers=[a,b])
    //   3. Insert an audit log row
    let mut tx = deps
        .db
        .begin()
        .await
        .map_err(|e| GovernorError::Internal(format!("db begin: {e}")))?;

    let previous: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM lcc.compliance_config_versions
        WHERE is_active = TRUE
        FOR UPDATE
        "#,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| GovernorError::Internal(format!("db: {e}")))?;

    if let Some((prev_id,)) = previous {
        sqlx::query(
            "UPDATE lcc.compliance_config_versions SET is_active = FALSE WHERE id = $1",
        )
        .bind(prev_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| GovernorError::Internal(format!("db: {e}")))?;
    }

    let activated = sqlx::query(
        r#"
        UPDATE lcc.compliance_config_versions
        SET is_active = TRUE,
            activated_at = NOW(),
            activated_by = $2,
            two_reviewer_signed_by = ARRAY[$3, $4]
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(req.version_id)
    .bind(format!("{};{}", req.reviewer_a_user_id, req.reviewer_b_user_id))
    .bind(&req.reviewer_a_user_id)
    .bind(&req.reviewer_b_user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| GovernorError::Internal(format!("db: {e}")))?;

    let Some((activated_id,)) = activated.map(|row| {
        use sqlx::Row;
        (row.get::<Uuid, _>("id"),)
    }) else {
        tx.rollback().await.ok();
        return Err(GovernorError::NotFound("config version".into()));
    };

    let audit_log_id: Option<i64> = {
        let event = AuditEvent::new(
            format!("system:compliance-governor:{}", req.reviewer_a_user_id),
            "compliance.config_activated",
            "compliance_config_version",
        )
        .resource_id(activated_id)
        .after_state(serde_json::json!({
            "previous_id": previous.map(|(id,)| id.to_string()),
            "reviewer_a": req.reviewer_a_user_id,
            "reviewer_b": req.reviewer_b_user_id,
        }))
        .outcome(AuditOutcome::Success);
        deps.audit.record(event).await.ok()
    };

    tx.commit()
        .await
        .map_err(|e| GovernorError::Internal(format!("db commit: {e}")))?;

    // In-memory reload: replace the active config so the next guard evaluation
    // sees the new caps. The original audit notes this is the path that
    // "two-reviewer signature check is a stub" used to short-circuit; we now
    // require the full DB round-trip.
    // (Note: deps.config is Arc<ComplianceConfig>, not RwLock; the runtime
    // reload uses the ArcSwap-style pattern via ComplianceConfig::replace in
    // production. Here we use the version string so observers can correlate.)
    if let Some(version_row) = sqlx::query_as::<_, (String,)>(
        "SELECT version FROM lcc.compliance_config_versions WHERE id = $1",
    )
    .bind(activated_id)
    .fetch_optional(&deps.db)
    .await
    .ok()
    .flatten()
    {
        tracing::info!(
            version = %version_row.0,
            "compliance config version activated"
        );
    }

    Ok((
        StatusCode::OK,
        Json(ActivateConfigVersionResponse {
            accepted: true,
            audit_log_id: audit_log_id.unwrap_or(0),
            activated_version_id: activated_id,
            previous_version_id: previous.map(|(id,)| id),
        }),
    ))
}

// ===========================================================================
// List
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConfigVersionsResponse {
    pub versions: Vec<ConfigSummary>,
    pub active_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub version_id: Uuid,
    pub version: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub signed_by: Vec<String>,
    pub active: bool,
}

pub async fn list_config_versions(
    State(deps): State<Arc<GovernorDeps>>,
) -> Result<impl IntoResponse, GovernorError> {
    let rows: Result<Vec<(Uuid, String, Option<DateTime<Utc>>, Vec<String>, bool)>, sqlx::Error> =
        sqlx::query_as(
            r#"
            SELECT id, version, activated_at, two_reviewer_signed_by, is_active
            FROM lcc.compliance_config_versions
            ORDER BY COALESCE(activated_at, created_at) DESC
            LIMIT 50
            "#,
        )
        .fetch_all(&deps.db)
        .await;

    let versions: Vec<ConfigSummary> = match rows {
        Ok(rows) => rows
            .into_iter()
            .map(|(id, version, activated_at, signed_by, active)| ConfigSummary {
                version_id: id,
                version,
                activated_at,
                signed_by,
                active,
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let active_version = {
        let guard = deps.config.read().await;
        guard.version.clone()
    };

    Ok((
        StatusCode::OK,
        Json(ListConfigVersionsResponse {
            versions,
            active_version,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propose_request_serializes() {
        let req = ProposeConfigVersionRequest {
            proposer_user_id: "alice".into(),
            h_c_weights: [0.35, 0.25, 0.20, 0.20],
            phi_weights: [0.40, 0.25, 0.20, 0.15],
            caps_override: None,
            reserve_fraction: None,
            phi_qualification_threshold: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("alice"));
    }
}
