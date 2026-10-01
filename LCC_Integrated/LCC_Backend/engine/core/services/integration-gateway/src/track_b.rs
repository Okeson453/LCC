//! Track B executor — sends actions to the user's browser-assist extension
//! via WSS, awaits human confirmation.

use crate::audit::{emit, IntegrationAuditEntry};
use crate::backoff::{BackoffTier, delay_for};
use crate::error::IntegrationError;
use crate::idempotency::IdempotencyStore;
use crate::permit::{verifier::PermitError, verifier::PermitVerifier, ReplayGuard, ReplayGuardError};
use crate::router::{route_with_post_target, Track};
use crate::state::IntegrationGatewayState;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use lcc_compliance::permit_token::PermitClaims;
use lcc_integrations::track_b::{
    BrowserExtensionMessage, ExtensionMessageKind, FillTarget,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;
use uuid::Uuid;

/// In-process map from correlation_id → oneshot that the WSS receiver
/// resolves when the human confirms or cancels.
type PendingConfirmation = Arc<tokio::sync::Mutex<std::collections::HashMap<Uuid, oneshot::Sender<ConfirmationOutcome>>>>;

#[derive(Debug, Clone)]
pub enum ConfirmationOutcome {
    Submitted { platform_id: Option<String> },
    Cancelled { reason: String },
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteActionRequest {
    pub permit_token: String,
    pub action_id: Uuid,
    pub member_id: Uuid,
    pub action_type: String,
    pub idempotency_key: String,
    pub target_contact_id: Option<Uuid>,
    pub target_post_id: Option<String>,
    pub fields: serde_json::Value,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteActionResponse {
    pub success: bool,
    pub track: Track,
    pub platform_response_id: Option<String>,
    pub cancelled_reason: Option<String>,
    pub duration_ms: u64,
}

pub async fn execute(
    state: Arc<IntegrationGatewayState>,
    pending: PendingConfirmation,
    req: ExecuteActionRequest,
) -> Result<ExecuteActionResponse, IntegrationError> {
    let start = Utc::now();
    let action_id = req.action_id;

    // 1. Verify permit_token. F-AUDIT-23: bind the action type too.
    //    F-AUDIT-24: the claims were previously bound to `_claims` and then
    //    discarded, so the `jti` needed for replay protection was thrown away.
    let claims: PermitClaims = state
        .permit_verifier
        .verify(&req.permit_token, &action_id, &req.member_id, &req.action_type)?;

    // 1b. F-AUDIT-22: single-use enforcement, identical to Track A.
    {
        let mut conn = state.redis.get().await?;
        ReplayGuard::claim(
            &mut conn,
            &claims.jti,
            ReplayGuard::dedupe_ttl(claims.exp),
        )
        .await
        .map_err(|e| {
            // F-AUDIT-22: this is the first construction of
            // `PermitError::Replay` in the repository — the variant existed
            // and was documented, but was never raised.
            match e {
                ReplayGuardError::Replay(jti) => {
                    IntegrationError::Permit(PermitError::Replay(jti))
                }
                ReplayGuardError::StoreUnavailable(msg) => IntegrationError::Permit(
                    PermitError::InvalidSignature(format!("replay store unavailable: {msg}")),
                ),
            }
        })?;
    }

    // 2. Idempotency check.
    if let Ok(Some(cached)) = state.idempotency.get(&req.idempotency_key).await {
        return Ok(ExecuteActionResponse {
            success: true,
            track: Track::TrackB,
            platform_response_id: cached
                .response_body
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from),
            cancelled_reason: None,
            duration_ms: 0,
        });
    }

    // 3. Route — Track B executor only handles Track B actions.
    let action_type = parse_action_type(&req.action_type)?;
    let track = route_with_post_target(action_type, false, &state.config);
    if track != Track::TrackB {
        return Err(IntegrationError::Config(
            "track_b executor received non-track_b action".into(),
        ));
    }

    // 4. Build the extension message.
    let target = map_to_fill_target(&req);
    let msg = BrowserExtensionMessage::fill_form(
        &req.member_id.to_string(),
        &req.action_id.to_string(),
        target,
        req.fields.clone(),
        req.timeout_seconds,
    );

    // 5. Register the oneshot receiver.
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(msg.correlation_id, tx);

    // 6. Push the message to the extension via WSS.
    let state_clone = state.clone();
    let msg_clone = msg.clone();
    tokio::spawn(async move {
        push_to_extension(state_clone, msg_clone).await;
    });

    // 7. Await human confirmation (with timeout).
    let timeout = std::time::Duration::from_secs(req.timeout_seconds);
    let outcome = match tokio::time::timeout(timeout, rx).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(_)) => ConfirmationOutcome::Cancelled { reason: "channel_closed".into() },
        Err(_) => {
            // Timeout — clean up pending entry.
            pending.lock().await.remove(&msg.correlation_id);
            ConfirmationOutcome::TimedOut
        }
    };

    // 8. Process outcome.
    let dur_ms = (Utc::now() - start).num_milliseconds().max(0) as u64;
    match outcome {
        ConfirmationOutcome::Submitted { platform_id } => {
            let body = serde_json::json!({
                "id": platform_id.clone().unwrap_or_default(),
                "sent_at": Utc::now().to_rfc3339(),
            });
            state.idempotency.put(&req.idempotency_key, 200, &body).await?;

            emit(
                &state.audit,
                IntegrationAuditEntry {
                    action_id,
                    member_id: req.member_id,
                    action_type: req.action_type.clone(),
                    track: "B".into(),
                    outcome: lcc_audit_client::AuditOutcome::Success,
                    reason: Some(format!("extension_submitted platform_id={platform_id:?}")),
                    restriction_signal: None,
                    duration_ms: dur_ms,
                },
            )
            .await
            .ok();

            Ok(ExecuteActionResponse {
                success: true,
                track: Track::TrackB,
                platform_response_id: platform_id,
                cancelled_reason: None,
                duration_ms: dur_ms,
            })
        }
        ConfirmationOutcome::Cancelled { reason } => {
            emit(
                &state.audit,
                IntegrationAuditEntry {
                    action_id,
                    member_id: req.member_id,
                    action_type: req.action_type.clone(),
                    track: "B".into(),
                    outcome: lcc_audit_client::AuditOutcome::Failed,
                    reason: Some(format!("cancelled: {reason}")),
                    restriction_signal: None,
                    duration_ms: dur_ms,
                },
            )
            .await
            .ok();

            Ok(ExecuteActionResponse {
                success: false,
                track: Track::TrackB,
                platform_response_id: None,
                cancelled_reason: Some(reason),
                duration_ms: dur_ms,
            })
        }
        ConfirmationOutcome::TimedOut => {
            emit(
                &state.audit,
                IntegrationAuditEntry {
                    action_id,
                    member_id: req.member_id,
                    action_type: req.action_type.clone(),
                    track: "B".into(),
                    outcome: lcc_audit_client::AuditOutcome::Failed,
                    reason: Some("extension_confirm_timeout".into()),
                    restriction_signal: None,
                    duration_ms: dur_ms,
                },
            )
            .await
            .ok();

            Err(IntegrationError::TrackB("human_confirm_timeout".into()))
        }
    }
}

async fn push_to_extension(
    state: Arc<IntegrationGatewayState>,
    msg: BrowserExtensionMessage,
) {
    // Look up the member's active WebSocket connection.
    let key = format!("ext:ws:{}", msg.member_id);
    let mut conn = match state.redis.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "redis unavailable for extension lookup");
            return;
        }
    };

    // Real impl: maintain a map of member_id → tokio-tungstenite::WebSocketStream
    // and push the JSON message. For this delivery, we record the message in
    // Redis as a list under `ext:pending:<member_id>` and let a separate
    // background pump drain it.
    let payload = msg.serialize().unwrap_or_default();
    let _: redis::RedisResult<i64> = redis::cmd("RPUSH")
        .arg(format!("ext:pending:{}", msg.member_id))
        .arg(payload)
        .query_async(&mut conn)
        .await;

    // Drop the key variable to silence unused warning when redis-rs is in use.
    let _ = key;
}

pub fn map_to_fill_target(req: &ExecuteActionRequest) -> FillTarget {
    match req.action_type.as_str() {
        "connection_request" => FillTarget::ConnectionRequestPage,
        "direct_message" | "sequence_step_send" => FillTarget::SendMessagePage,
        "post_publish" => FillTarget::CreatePostPage,
        "comment" => FillTarget::CommentComposer,
        _ => FillTarget::SearchPage,
    }
}

fn parse_action_type(s: &str) -> Result<lcc_compliance::action::ActionType, IntegrationError> {
    serde_json::from_str(&format!("\"{s}\""))
        .map_err(|e| IntegrationError::Config(format!("bad action_type: {e}")))
}

/// WSS message handler — invoked by the WSS server when the extension sends
/// a `ActionSubmitted` / `ActionCancelled` frame.
pub async fn handle_extension_frame(
    pending: PendingConfirmation,
    raw: &str,
) -> Result<(), IntegrationError> {
    let msg = BrowserExtensionMessage::deserialize(raw)
        .map_err(|e| IntegrationError::TrackB(format!("deserialize: {e}")))?;

    match &msg.message {
        ExtensionMessageKind::ActionSubmitted { platform_response_id, .. } => {
            let tx = pending.lock().await.remove(&msg.correlation_id);
            if let Some(tx) = tx {
                let _ = tx.send(ConfirmationOutcome::Submitted {
                    platform_id: platform_response_id.clone(),
                });
            }
        }
        ExtensionMessageKind::ActionCancelled { reason } => {
            let tx = pending.lock().await.remove(&msg.correlation_id);
            if let Some(tx) = tx {
                let _ = tx.send(ConfirmationOutcome::Cancelled { reason: reason.clone() });
            }
        }
        ExtensionMessageKind::Pong
        | ExtensionMessageKind::Ping
        | ExtensionMessageKind::Ready
        | ExtensionMessageKind::FillForm { .. }
        | ExtensionMessageKind::ActionEdited { .. } => {
            // No-op for now.
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_to_fill_target_for_connection_request() {
        let req = ExecuteActionRequest {
            permit_token: "x".into(),
            action_id: Uuid::now_v7(),
            member_id: Uuid::now_v7(),
            action_type: "connection_request".into(),
            idempotency_key: "k".into(),
            target_contact_id: None,
            target_post_id: None,
            fields: serde_json::json!({}),
            timeout_seconds: 60,
        };
        assert!(matches!(map_to_fill_target(&req), FillTarget::ConnectionRequestPage));
    }

    #[test]
    fn map_to_fill_target_for_dm() {
        let req = ExecuteActionRequest {
            permit_token: "x".into(),
            action_id: Uuid::now_v7(),
            member_id: Uuid::now_v7(),
            action_type: "direct_message".into(),
            idempotency_key: "k".into(),
            target_contact_id: None,
            target_post_id: None,
            fields: serde_json::json!({}),
            timeout_seconds: 60,
        };
        assert!(matches!(map_to_fill_target(&req), FillTarget::SendMessagePage));
    }

    #[test]
    fn parse_action_type_round_trip() {
        let a = parse_action_type("connection_request").unwrap();
        assert_eq!(a, lcc_compliance::action::ActionType::ConnectionRequest);
    }
}
