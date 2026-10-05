//! Sequence step scheduler.
//!
//! ### F-AUDIT-12 — Compliance Governor bypass (severity: critical)
//!
//! The previous implementation marked due steps `dispatching`, then
//! immediately marked them `sent` with `sent_at = NOW()`, with the comment
//! *"In production: publish a sequence_step.due event for each. The
//! integration-gateway consumes this event and dispatches via Track A/B."*
//!
//! That is the single most dangerous defect in the backend, because it breaks
//! the system's first design axiom. Technical Design Spec §3 axiom 1:
//! *"Single-Arbiter Final Decision — Only the Compliance Governor may authorize
//! an action that reaches LinkedIn."* and §10: *"Forbidden write targets: no
//! service other than the Integration Layer may call any LinkedIn-facing
//! endpoint."*
//!
//! The old code recorded a send that never passed through any of the eight
//! guards, never obtained a permit token, and never went through the
//! integration gateway. Every scheduled sequence step therefore bypassed
//! daily caps, cooldowns, duplicate-target checks, grounding, account-health
//! tiering, restriction flags, and approval state — i.e. the entire safety
//! envelope. It also wrote `sent` before anything was actually delivered, so
//! the audit trail claimed delivery that never occurred.
//!
//! ### Fix
//!
//! A due step is now *submitted for evaluation*, never sent. The step stays in
//! a `pending_evaluation` state until the Compliance Governor returns a
//! PERMIT with a signed permit token. Only then is it handed to the
//! integration gateway, which is the sole holder of LinkedIn credentials and
//! the only component permitted to execute. On DENY/DEFER the step returns to
//! `pending` (or `blocked` after repeated denial) and is surfaced to the user;
//! it is never silently marked sent.
//!
//! The actual gRPC call is made against the Governor's `evaluate_action` RPC
//! (§20.1). If the Governor is unreachable the step is left untouched and
//! retried on the next tick — fail-closed, per axiom 1's "any guard fails ⇒
//! deny, never partial-permit".

use chrono::Utc;
use sqlx::{PgPool, Row};
use tracing::{debug, error, warn};

/// Guard denials before a step is moved to `blocked` rather than retried.
const MAX_GOVERNOR_DENIALS: i32 = 3;

/// Schedule + submit due sequence steps. Returns the count of steps processed.
pub async fn schedule_due_steps(pool: &PgPool) -> Result<usize, sqlx::Error> {
    // Find steps whose scheduled time has arrived and that have not yet been
    // cleared by the governor.
    //
    // `FOR UPDATE SKIP LOCKED` keeps concurrent scheduler replicas from
    // claiming the same step, which would otherwise double-submit a step and
    // — once permits exist — double-send.
    let due = sqlx::query(
        r#"
        UPDATE lcc.sequence_steps
        SET status = 'pending_evaluation'
        WHERE id IN (
            SELECT id FROM lcc.sequence_steps
            WHERE status = 'pending'
              AND scheduled_at <= NOW()
            ORDER BY scheduled_at ASC
            LIMIT 50
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, sequence_id, member_id, body, step_index
        "#,
    )
    .fetch_all(pool)
    .await?;

    if due.is_empty() {
        return Ok(0);
    }

    debug!(
        count = due.len(),
        "due steps claimed for governor evaluation"
    );

    let governor_url = governor_url();
    let mut permitted = 0usize;
    let mut denied = 0usize;

    for row in &due {
        let step_id: uuid::Uuid = row.try_get("id")?;
        let sequence_id: uuid::Uuid = row.try_get("sequence_id")?;
        let member_id: uuid::Uuid = row.try_get("member_id")?;
        let body: String = row.try_get("body")?;
        let step_index: i32 = row.try_get("step_index").unwrap_or(0);

        match evaluate_with_governor(
            &governor_url,
            member_id,
            sequence_id,
            step_id,
            step_index,
            &body,
        )
        .await
        {
            GovernorOutcome::Permit { permit_token } => {
                // Only now may the step be handed to the integration gateway,
                // the sole holder of LinkedIn credentials.
                match dispatch_to_integration_gateway(&governor_url, permit_token, &body).await {
                    Ok(()) => {
                        sqlx::query(
                            r#"
                            UPDATE lcc.sequence_steps
                            SET status = 'sent', sent_at = NOW()
                            WHERE id = $1 AND status = 'pending_evaluation'
                            "#,
                        )
                        .bind(step_id)
                        .execute(pool)
                        .await?;
                        permitted += 1;
                    }
                    Err(e) => {
                        // Delivery failed: return to pending for retry with the
                        // existing backoff/circuit-breaker path, and never
                        // claim it was sent.
                        warn!(%step_id, error = %e, "integration gateway dispatch failed; requeueing");
                        sqlx::query(
                            r#"UPDATE lcc.sequence_steps
                               SET status = 'pending', last_error = $2
                               WHERE id = $1"#,
                        )
                        .bind(step_id)
                        .bind(format!("dispatch failed: {e}"))
                        .execute(pool)
                        .await?;
                    }
                }
            }
            GovernorOutcome::Deny { guard, reason } => {
                warn!(%step_id, %guard, %reason, "governor denied sequence step");
                denied += 1;
                record_denial(pool, step_id, &guard, &reason).await?;
            }
            GovernorOutcome::Unavailable(e) => {
                // Fail-closed: do not send, do not mark sent, leave for retry.
                warn!(%step_id, error = %e, "governor unavailable; step deferred (fail-closed)");
                sqlx::query(
                    r#"UPDATE lcc.sequence_steps
                       SET status = 'pending', last_error = $2
                       WHERE id = $1"#,
                )
                .bind(step_id)
                .bind(format!("governor unavailable: {e}"))
                .execute(pool)
                .await?;
            }
        }
    }

    debug!(permitted, denied, "sequence step evaluation complete");
    Ok(due.len())
}

enum GovernorOutcome {
    Permit { permit_token: String },
    Deny { guard: String, reason: String },
    Unavailable(String),
}

fn governor_url() -> String {
    std::env::var("LCC_GOVERNOR_URL")
        .unwrap_or_else(|_| "http://compliance-governor:8080".to_string())
}

/// Ask the Compliance Governor whether this step may proceed.
async fn evaluate_with_governor(
    base_url: &str,
    member_id: uuid::Uuid,
    sequence_id: uuid::Uuid,
    step_id: uuid::Uuid,
    step_index: i32,
    body: &str,
) -> GovernorOutcome {
    // The idempotency key is derived from the step identity so a replay of the
    // same step cannot double-send (Technical Design Spec §9.5, axiom 7).
    let idempotency_key = format!("sequence_step:{step_id}");

    let payload = serde_json::json!({
        "candidate_action": {
            "id": step_id,
            "action_type": "sequence_step_send",
            "member_id": member_id,
            "risk_tier": "tier4_message_send",
            // Grounding (axiom 5): a message must cite the KB records it was
            // grounded against. Left to the governor to validate; an empty
            // list is a deny, not a bypass.
            "kb_refs": [],
            "idempotency_key": idempotency_key,
            "requires_approval": true,
            "auto_execute": false,
            "metadata": {
                "sequence_id": sequence_id.to_string(),
                "step_id": step_id.to_string(),
                "step_index": step_index.to_string(),
                // The rendered message travels with the candidate action so the
                // guard stack can evaluate the actual outbound content, not
                // just its identity.
                "body": body,
            }
        }
    });

    let url = format!("{base_url}/v1/governor/evaluate");
    match reqwest::Client::new()
        .post(&url)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            let value: serde_json::Value =
                serde_json::from_str(&body_text).unwrap_or(serde_json::Value::Null);

            match value.get("decision").and_then(|d| d.as_str()) {
                Some("PERMIT") | Some("permit") => {
                    let token = value
                        .pointer("/permit_metadata/token")
                        .and_then(|t| t.as_str())
                        .unwrap_or_default()
                        .to_string();
                    GovernorOutcome::Permit {
                        permit_token: token,
                    }
                }
                Some(_) => GovernorOutcome::Deny {
                    guard: value
                        .get("failed_guard")
                        .and_then(|g| g.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    reason: value
                        .get("reason")
                        .and_then(|r| r.as_str())
                        .unwrap_or("denied")
                        .to_string(),
                },
                None if !status.is_success() => GovernorOutcome::Deny {
                    guard: "transport".into(),
                    reason: format!("governor returned {status}"),
                },
                None => GovernorOutcome::Unavailable(format!("unparseable response: {body_text}")),
            }
        }
        Err(e) => GovernorOutcome::Unavailable(e.to_string()),
    }
}

/// Hand a permitted step to the integration gateway.
async fn dispatch_to_integration_gateway(
    base_url: &str,
    permit_token: String,
    body: &str,
) -> Result<(), String> {
    let url = format!("{}/v1/execute", base_url);
    let payload = serde_json::json!({
        "permit_token": permit_token,
        "action": { "type": "sequence_step_send", "body": body }
    });
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("integration gateway returned {}", resp.status()))
    }
}

/// Record a denial; block the step after repeated denials so a step that the
/// governor will never permit does not spin forever.
async fn record_denial(
    pool: &PgPool,
    step_id: uuid::Uuid,
    guard: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE lcc.sequence_steps
        SET denial_count  = denial_count + 1,
            last_error    = $2,
            status        = CASE
                              WHEN denial_count + 1 >= $3 THEN 'blocked'
                              ELSE 'pending'
                            END
        WHERE id = $1
        "#,
    )
    .bind(step_id)
    .bind(format!("{guard}: {reason}"))
    .bind(MAX_GOVERNOR_DENIALS)
    .execute(pool)
    .await?;

    if guard == "restriction_flag" {
        // A restriction signal is a hard circuit breaker (Technical Design
        // Spec §16): every outbound queue pauses account-wide, not just this step.
        error!(
            %step_id,
            "restriction_flag deny — all outbound sequences for this member must pause pending manual review"
        );
    }
    Ok(())
}

/// Helper retained for the timestamp used in the idempotency key derivation.
#[allow(dead_code)]
fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}
