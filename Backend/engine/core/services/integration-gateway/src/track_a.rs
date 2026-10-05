//! Track A executor — wraps the LinkedIn API clients with permit_token
//! validation, idempotency, circuit breaker, restriction detection.

use crate::audit::{emit, IntegrationAuditEntry};
use crate::backoff::{delay_for, BackoffTier};
use crate::circuit_breaker::CircuitBreaker;
use crate::error::IntegrationError;
use crate::permit::{verifier::PermitError, ReplayGuard, ReplayGuardError};
use crate::restriction::{detect_status_and_body, RestrictionSignal};
use crate::router::{route_with_post_target, Track};
use crate::state::IntegrationGatewayState;
use chrono::Utc;
use lcc_compliance::action::ActionType;
use lcc_compliance::permit_token::PermitClaims;
use lcc_integrations::track_a::{jobs_client, oauth_client, profile_client, share_client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteActionRequest {
    pub permit_token: String,
    pub action_id: Uuid,
    pub member_id: Uuid,
    pub action_type: String,
    pub idempotency_key: String,
    pub is_organization: bool,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteActionResponse {
    pub success: bool,
    pub track: Track,
    pub platform_response_id: Option<String>,
    pub restriction_signal: RestrictionSignal,
    pub duration_ms: u64,
    pub attempt_count: u32,
    pub error: Option<String>,
}

/// Execute an action through Track A. Returns the response on success;
/// propagates restriction-signal failures so the gateway can mark the account.
pub async fn execute(
    state: Arc<IntegrationGatewayState>,
    req: ExecuteActionRequest,
) -> Result<ExecuteActionResponse, IntegrationError> {
    let start = Utc::now();
    let action_id = req.action_id;

    // 1. Verify permit_token. F-AUDIT-23: the requested action type is now
    //    bound into verification, so a permit minted for one action type
    //    cannot authorise a different one.
    let claims: PermitClaims = state.permit_verifier.verify(
        &req.permit_token,
        &action_id,
        &req.member_id,
        &req.action_type,
    )?;

    // 1b. F-AUDIT-22: single-use enforcement. A permit is valid for exactly
    //     one execution; a second presentation of the same `jti` is a replay
    //     and is rejected before any outbound call is made. Fails closed on a
    //     Redis error, matching the governor's deny-on-guard-failure posture.
    {
        let mut conn = state.redis.get().await?;
        ReplayGuard::claim(&mut conn, &claims.jti, ReplayGuard::dedupe_ttl(claims.exp))
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

    // 2. Check idempotency store first.
    if let Ok(Some(cached)) = state.idempotency.get(&req.idempotency_key).await {
        tracing::info!(key = %req.idempotency_key, "idempotent replay");
        return Ok(ExecuteActionResponse {
            success: true,
            track: Track::TrackA,
            platform_response_id: cached
                .response_body
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from),
            restriction_signal: RestrictionSignal::None,
            duration_ms: 0,
            attempt_count: 1,
            error: None,
        });
    }

    // 3. Check circuit breaker.
    let breaker = CircuitBreaker::new("linkedin", action_endpoint(&req.action_type));
    {
        let mut conn = state.redis.get().await?;
        if !breaker.should_allow(&mut conn).await? {
            return Err(IntegrationError::CircuitOpen {
                provider: "linkedin".into(),
                endpoint: action_endpoint(&req.action_type),
            });
        }
    }

    // 3b. Per-endpoint budget. The governor caps what a member may do, but
    // the gateway is what actually talks to LinkedIn: without this check the
    // aggregate of many members' actions can still exceed the provider's
    // limits, which is exactly how an account gets restricted.
    let endpoint = action_endpoint(&req.action_type);
    state
        .rate_limiter
        .check_and_increment(&endpoint)
        .map_err(|reason| IntegrationError::RateLimitExceeded { endpoint, reason })?;

    // 4. Resolve track + member's access token from Vault.
    let action_type = parse_action_type(&req.action_type)?;
    let track = route_with_post_target(action_type, req.is_organization, &state.config);

    // 5. Execute (with retries + exponential backoff).
    let mut last_err: Option<String> = None;
    let mut last_signal = RestrictionSignal::None;
    let mut attempt = 1u32;
    // Source §39.2: publishing retries on the heavy tier (base 5s, cap 80s).
    // Everything else uses the standard tier (base 2s, cap 60s). A publish is
    // the most consequential action — retrying it gently matters more than
    // retrying a like quickly.
    let backoff_tier = backoff_tier_for(&req.action_type);
    let max_retries = backoff_tier.max_retries();

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| IntegrationError::Config(e.to_string()))?;

    // Get member's access token (the only place this Vault path is read).
    let access_token = state
        .vault
        .read(&lcc_security::vault::SecretRef::new(format!(
            "secret/linkedin/oauth/{}/access_token",
            req.member_id
        )))
        .await?;

    while attempt <= max_retries {
        let exec_result = match track {
            Track::TrackA => execute_track_a(&http, &req, &access_token).await,
            Track::TrackB => {
                // This branch shouldn't be reached from the Track A executor;
                // Track B has its own executor. Return an error if reached.
                return Err(IntegrationError::Config(
                    "track_a received track_b action".into(),
                ));
            }
        };

        match exec_result {
            Ok(platform_resp_id) => {
                // Success — record + cache.
                let body =
                    serde_json::json!({"id": platform_resp_id, "sent_at": Utc::now().to_rfc3339()});
                state
                    .idempotency
                    .put(&req.idempotency_key, 200, &body)
                    .await?;

                let dur_ms = (Utc::now() - start).num_milliseconds().max(0) as u64;
                emit(
                    &state.audit,
                    IntegrationAuditEntry {
                        action_id,
                        member_id: req.member_id,
                        action_type: req.action_type.clone(),
                        track: track.as_str().into(),
                        outcome: lcc_audit_client::AuditOutcome::Success,
                        reason: Some(format!("platform_id={platform_resp_id}")),
                        restriction_signal: None,
                        duration_ms: dur_ms,
                    },
                )
                .await
                .ok();

                // Reset breaker.
                let mut conn = state.redis.get().await?;
                breaker.record_success(&mut conn).await?;

                return Ok(ExecuteActionResponse {
                    success: true,
                    track,
                    platform_response_id: Some(platform_resp_id),
                    restriction_signal: RestrictionSignal::None,
                    duration_ms: dur_ms,
                    attempt_count: attempt,
                    error: None,
                });
            }
            Err(e @ IntegrationError::LinkedIn { .. }) => {
                // F-72: status is now carried on the error variant. Run the
                // detector against the *real* status, falling back to body-only
                // when status is unknown (e.g., transport-level error before
                // we ever saw an HTTP response).
                let (signal, _) = detect_status_and_body(e.http_status(), e.body_text());
                if signal != RestrictionSignal::None {
                    // Hard pause; no retry (axiom 6).
                    last_signal = signal;
                    last_err = Some(format!("restriction: {}", signal.as_str()));
                    break;
                }

                // Increment breaker.
                let mut conn = state.redis.get().await?;
                let _ = breaker.record_failure(&mut conn).await;

                last_err = Some(e.body_text().to_string());
                attempt += 1;
                if attempt > max_retries {
                    break;
                }
                tokio::time::sleep(delay_for(attempt - 1, backoff_tier)).await;
            }
            Err(IntegrationError::LinkedInLegacy(msg)) => {
                // Pre-F-72 callers — preserve fallback detection by body text.
                let (signal, _) = detect_status_and_body(0, &msg);
                if signal != RestrictionSignal::None {
                    last_signal = signal;
                    last_err = Some(format!("restriction: {}", signal.as_str()));
                    break;
                }
                last_err = Some(msg);
                attempt += 1;
                if attempt > max_retries {
                    break;
                }
                tokio::time::sleep(delay_for(attempt - 1, backoff_tier)).await;
            }
            Err(e) => return Err(e),
        }
    }

    // Exhausted retries.
    let dur_ms = (Utc::now() - start).num_milliseconds().max(0) as u64;
    let outcome = if last_signal != RestrictionSignal::None {
        lcc_audit_client::AuditOutcome::Denied
    } else {
        lcc_audit_client::AuditOutcome::Failed
    };
    emit(
        &state.audit,
        IntegrationAuditEntry {
            action_id,
            member_id: req.member_id,
            action_type: req.action_type.clone(),
            track: track.as_str().into(),
            outcome,
            reason: last_err.clone(),
            restriction_signal: if last_signal != RestrictionSignal::None {
                Some(last_signal)
            } else {
                None
            },
            duration_ms: dur_ms,
        },
    )
    .await
    .ok();

    Ok(ExecuteActionResponse {
        success: false,
        track,
        platform_response_id: None,
        restriction_signal: last_signal,
        duration_ms: dur_ms,
        attempt_count: attempt - 1,
        error: last_err,
    })
}

async fn execute_track_a(
    http: &reqwest::Client,
    req: &ExecuteActionRequest,
    access_token: &str,
) -> Result<String, IntegrationError> {
    let action_type = parse_action_type(&req.action_type)?;

    match action_type {
        ActionType::PostPublish => {
            let body: share_client::UgcPostRequest = serde_json::from_value(req.payload.clone())?;
            let resp =
                share_client::publish_ugc_post(http, access_token, &body, &req.idempotency_key)
                    .await?;
            Ok(resp.id)
        }
        _ => Err(IntegrationError::linkedin_unknown(format!(
            "action_type={action_type:?} not supported by track_a"
        ))),
    }
}

fn parse_action_type(s: &str) -> Result<ActionType, IntegrationError> {
    serde_json::from_str(&format!("\"{s}\""))
        .map_err(|e| IntegrationError::Config(format!("bad action_type: {e}")))
}

/// Which backoff tier an action retries on (Source §39.2).
fn backoff_tier_for(action_type: &str) -> BackoffTier {
    match action_type {
        "post_publish" | "ugc_post" | "organization_page_post_publish" => BackoffTier::Heavy,
        _ => BackoffTier::Standard,
    }
}

fn action_endpoint(action_type: &str) -> String {
    match action_type {
        "post_publish" | "ugc_post" => "ugc_post".into(),
        "userinfo" | "profile_read" => "userinfo".into(),
        "jobs_search" | "job_search" => "jobs_search".into(),
        _ => action_type.to_string(),
    }
}

// Keep imports clean — we don't use these directly in this file but they're
// part of the public Track A surface that the executor can route to.
#[allow(unused_imports)]
use {jobs_client as _, oauth_client as _, profile_client as _};

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_type_round_trips() {
        let a = parse_action_type("post_publish").unwrap();
        assert_eq!(a, ActionType::PostPublish);
    }

    #[test]
    fn parse_unknown_returns_error() {
        assert!(parse_action_type("not_a_real_action").is_err());
    }

    #[test]
    fn publish_uses_the_heavy_backoff_tier() {
        assert_eq!(backoff_tier_for("post_publish"), BackoffTier::Heavy);
        assert_eq!(
            backoff_tier_for("organization_page_post_publish"),
            BackoffTier::Heavy
        );
        // A heavy retry waits longer than a standard one for the same index.
        assert!(backoff_tier_for("post_publish").base_ms() > backoff_tier_for("like").base_ms());
        assert_eq!(backoff_tier_for("like"), BackoffTier::Standard);
    }

    #[test]
    fn action_endpoint_normalizes() {
        assert_eq!(action_endpoint("post_publish"), "ugc_post");
        assert_eq!(action_endpoint("jobs_search"), "jobs_search");
    }
}
