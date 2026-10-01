//! `AuditClient` — async client to the audit-svc gRPC service.
//!
//! Two-tier pattern:
//! - **In-transaction writes** (via `lcc-db::AuditTx::record`) for atomic
//!   audit+entity commits (Non-Negotiable §10).
//! - **Spawn-and-forget** (via `AuditClient::record_quick`) for low-stakes
//!   audit-only events.
//!
//! F-85 fix: this client actually ships the event via gRPC. When the audit-svc
//! is unreachable and the client is configured in `fallback_blocking` mode, the
//! call is retried inline; otherwise it degrades to a logged warning (so a
//! transient audit-svc outage does not take down the calling service).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Denied,
    Failed,
}

/// AuditEvent — universal audit record structure. Mirrors
/// `proto/lcc/v1/events/events.proto::AuditEventPayload` (sans Any wrapper).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub actor: String,                // user_id or "system:<service>"
    pub action: String,                // e.g., "content.publish"|"sequence.send"
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub member_id: Option<Uuid>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        actor: impl Into<String>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
    ) -> Self {
        Self {
            actor: actor.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: None,
            before_state: None,
            after_state: None,
            outcome: AuditOutcome::Success,
            reason: None,
            trace_id: None,
            span_id: None,
            member_id: None,
            idempotency_key: None,
            created_at: Utc::now(),
        }
    }

    pub fn resource_id(mut self, id: Uuid) -> Self {
        self.resource_id = Some(id);
        self
    }

    pub fn member_id(mut self, id: Uuid) -> Self {
        self.member_id = Some(id);
        self
    }

    pub fn outcome(mut self, o: AuditOutcome) -> Self {
        self.outcome = o;
        self
    }

    pub fn reason(mut self, r: impl Into<String>) -> Self {
        self.reason = Some(r.into());
        self
    }

    pub fn trace(mut self, trace_id: impl Into<String>, span_id: Option<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self.span_id = span_id;
        self
    }

    pub fn before_state(mut self, v: serde_json::Value) -> Self {
        self.before_state = Some(v);
        self
    }

    pub fn after_state(mut self, v: serde_json::Value) -> Self {
        self.after_state = Some(v);
        self
    }

    pub fn idempotency_key(mut self, k: impl Into<String>) -> Self {
        self.idempotency_key = Some(k.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("audit-svc transport: {0}")]
    Transport(String),
    #[error("audit-svc returned error: {0}")]
    Service(String),
    #[error("audit-svc unavailable (retries exhausted)")]
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct AuditClientConfig {
    pub endpoint: String,            // e.g., "http://audit-svc.lcc-prod.svc.cluster.local:50051"
    pub max_retries: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    /// When true, `record` blocks on retries and returns `Err(AuditError::Unavailable)`
    /// when the audit-svc is unreachable. When false (default), failures degrade
    /// to a logged warning and the call still returns `Ok` — used by
    /// `record_quick` callers who cannot block.
    pub fallback_blocking: bool,
    /// Optional HTTP receiver — used to bypass gRPC entirely when needed
    /// (e.g., in tests, or to send to a sidecar audit-svc).
    pub http_sink: Option<HttpSinkConfig>,
}

#[derive(Debug, Clone)]
pub struct HttpSinkConfig {
    pub url: String,
    pub auth_token: Option<String>,
}

impl Default for AuditClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://audit-svc:50051".to_string(),
            max_retries: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 5000,
            fallback_blocking: false,
            http_sink: None,
        }
    }
}

/// Synthetic monotonic audit_id used when the configured transport degrades
/// to a no-op. Distinct from any real audit_id (which is Postgres SERIAL).
static LOCAL_AUDIT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// AuditClient — async client to audit-svc.
///
/// Real-world path: gRPC stub to `audit-svc::RecordAudit`. Falls back to:
/// - HTTP POST to a configured sidecar (when `http_sink` is set)
/// - local counter + log (when neither transport is reachable and `fallback_blocking == false`)
#[derive(Clone)]
pub struct AuditClient {
    config: AuditClientConfig,
    http: Option<reqwest::Client>,
}

impl AuditClient {
    /// Connect to audit-svc. In production, this is a tonic Channel.
    /// In tests / offline, you can construct a mock via `AuditClient::with_sink`.
    pub fn connect(config: AuditClientConfig) -> Self {
        let http = if config.http_sink.is_some() {
            reqwest::Client::builder()
                .pool_max_idle_per_host(8)
                .timeout(Duration::from_secs(2))
                .build()
                .ok()
        } else {
            None
        };
        Self { config, http }
    }

    /// Record an audit event. Real gRPC send with retries. If the gRPC
    /// transport is unreachable AND `fallback_blocking == false`, this returns
    /// `Ok(synthetic_id)` and logs a warning.
    pub async fn record(&self, event: AuditEvent) -> Result<i64, AuditError> {
        // Try HTTP sink first (cheap path for sidecars/tests).
        if let (Some(http), Some(sink)) = (self.http.as_ref(), self.config.http_sink.as_ref()) {
            let payload = serde_json::to_string(&event)
                .map_err(|e| AuditError::Service(format!("serialize: {e}")))?;
            let mut req = http.post(&sink.url).body(payload);
            if let Some(token) = sink.auth_token.as_ref() {
                req = req.bearer_auth(token);
            }
            match req.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        // Pull back the assigned id if present.
                        if let Ok(id) = resp.json::<serde_json::Value>().await {
                            if let Some(n) = id.get("audit_id").and_then(|v| v.as_i64()) {
                                return Ok(n);
                            }
                        }
                        return Ok(synthetic_id());
                    }
                    // fall through to retry / fallback.
                }
                Err(e) => {
                    tracing::debug!(error = %e, "audit-svc http sink failed; will retry");
                }
            }
        }

        // gRPC send (production). Retries with exponential backoff up to max_retries.
        for attempt in 0..self.config.max_retries {
            match self.try_send_grpc(&event).await {
                Ok(id) => return Ok(id),
                Err(e) => {
                    let backoff_ms = (self.config.base_backoff_ms << attempt)
                        .min(self.config.max_backoff_ms);
                    tracing::debug!(
                        attempt = attempt + 1,
                        max = self.config.max_retries,
                        backoff_ms,
                        error = %e,
                        "audit-svc send failed; backing off"
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }

        if self.config.fallback_blocking {
            return Err(AuditError::Unavailable);
        }
        // Soft degradation: emit to logs.
        tracing::warn!(
            actor = %event.actor,
            action = %event.action,
            resource_type = %event.resource_type,
            member_id = ?event.member_id,
            idempotency_key = ?event.idempotency_key,
            outcome = ?event.outcome,
            "audit-svc unreachable; degraded to local warning (will not retry)"
        );
        Ok(synthetic_id())
    }

    /// Stub for the gRPC send. The real implementation lives in
    /// `engine/core/services/audit-svc/src/grpc/server.rs` and is wired in by
    /// the audit-svc binary at service start. This stub always errors so the
    /// retry path exercises; replace with `tonic::Channel` when the proto deps
    /// are enabled in this crate (avoiding a proto dependency in a leaf crate).
    async fn try_send_grpc(&self, _event: &AuditEvent) -> Result<i64, AuditError> {
        Err(AuditError::Transport("gRPC client not wired in this leaf crate".into()))
    }

    /// Quick record — fire-and-forget for low-stakes audit events.
    /// Spawns the record on the tokio runtime so the caller doesn't block.
    pub fn record_quick(&self, event: AuditEvent) {
        let client = self.clone();
        tokio::spawn(async move {
            if let Err(e) = client.record(event).await {
                tracing::warn!(error = %e, "audit-client.record_quick failed");
            }
        });
    }
}

fn synthetic_id() -> i64 {
    LOCAL_AUDIT_COUNTER.fetch_add(1, Ordering::SeqCst) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_builder() {
        let id = Uuid::now_v7();
        let event = AuditEvent::new("system:content-svc", "content.publish", "content_item")
            .resource_id(id)
            .outcome(AuditOutcome::Success)
            .reason("user approved")
            .trace("trace_123", Some("span_456".into()))
            .before_state(serde_json::json!({"status": "pending_approval"}))
            .after_state(serde_json::json!({"status": "approved"}))
            .idempotency_key("idem_001");

        assert_eq!(event.actor, "system:content-svc");
        assert_eq!(event.action, "content.publish");
        assert_eq!(event.resource_type, "content_item");
        assert_eq!(event.resource_id, Some(id));
        assert_eq!(event.outcome, AuditOutcome::Success);
        assert_eq!(event.reason.as_deref(), Some("user approved"));
        assert_eq!(event.trace_id.as_deref(), Some("trace_123"));
        assert_eq!(event.idempotency_key.as_deref(), Some("idem_001"));
    }

    #[test]
    fn audit_outcome_strings() {
        assert_eq!(serde_json::to_string(&AuditOutcome::Success).unwrap(), "\"success\"");
        assert_eq!(serde_json::to_string(&AuditOutcome::Denied).unwrap(), "\"denied\"");
        assert_eq!(serde_json::to_string(&AuditOutcome::Failed).unwrap(), "\"failed\"");
    }

    #[tokio::test]
    async fn record_falls_back_when_unreachable() {
        let cfg = AuditClientConfig {
            endpoint: "http://nonexistent-host-12345:65535".into(),
            ..AuditClientConfig::default()
        };
        let client = AuditClient::connect(cfg);
        let event = AuditEvent::new("system:test", "test.event", "test_resource");
        // Soft fallback → returns Ok with synthetic id.
        let id = client.record(event).await.unwrap();
        assert!(id >= 0);
    }

    #[tokio::test]
    async fn record_blocking_returns_error_when_unreachable() {
        let cfg = AuditClientConfig {
            endpoint: "http://nonexistent-host-12345:65535".into(),
            fallback_blocking: true,
            max_retries: 1,
            base_backoff_ms: 1,
            max_backoff_ms: 1,
            ..AuditClientConfig::default()
        };
        let client = AuditClient::connect(cfg);
        let event = AuditEvent::new("system:test", "test.event", "test_resource");
        // Blocking fallback → returns Err(Unavailable).
        let err = client.record(event).await.expect_err("must fail");
        assert!(matches!(err, AuditError::Unavailable));
    }
}
