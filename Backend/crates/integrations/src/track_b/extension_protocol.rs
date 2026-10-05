//! Browser-assist extension messaging protocol (WSS).
//!
//! Protocol: JSON over WSS. Each message has a `kind`, a `payload`, and a
//! `correlation_id` for matching responses to requests.
//!
//! ### Message flow (high-level)
//! ```text
//! integration-gateway → extension: {"kind":"fill_form","payload":{...}}
//! extension → user:          user reviews and clicks "Confirm"
//! extension → gateway:       {"kind":"action_submitted","correlation_id":...}
//! ```
//!
//! The gateway NEVER submits the form on its own; it always waits for the
//! human confirmation frame before persisting `sent_at`.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TrackBError {
    #[error("websocket error: {0}")]
    WebSocket(String),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("timeout waiting for human confirmation")]
    HumanConfirmTimeout,
    #[error("member extension offline")]
    ExtensionOffline,
    #[error("serialization: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtensionMessageKind {
    /// Server → extension: fill a form with the supplied payload
    FillForm {
        target: FillTarget,
        fields: serde_json::Value,
        wait_for_human_confirm: bool,
        timeout_seconds: u64,
    },
    /// Extension → server: human confirmed the action
    ActionSubmitted {
        target: FillTarget,
        platform_response_id: Option<String>,
    },
    /// Extension → server: human cancelled
    ActionCancelled { reason: String },
    /// Extension → server: human is editing the payload
    ActionEdited { edited_fields: serde_json::Value },
    /// Server → extension: heartbeat / keepalive
    Ping,
    /// Extension → server: heartbeat ack
    Pong,
    /// Server → extension: page navigated / ready to receive next
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FillTarget {
    ConnectionRequestPage,
    SendMessagePage,
    CreatePostPage,
    CommentComposer,
    SearchPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserExtensionMessage {
    pub correlation_id: Uuid,
    pub member_id: String,
    pub action_id: String, // permits ↔ action binding
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: ExtensionMessageKind,
}

impl BrowserExtensionMessage {
    pub fn new(
        member_id: impl Into<String>,
        action_id: impl Into<String>,
        message: ExtensionMessageKind,
    ) -> Self {
        Self {
            correlation_id: Uuid::now_v7(),
            member_id: member_id.into(),
            action_id: action_id.into(),
            timestamp: chrono::Utc::now(),
            message,
        }
    }

    pub fn fill_form(
        member_id: &str,
        action_id: &str,
        target: FillTarget,
        fields: serde_json::Value,
        timeout_seconds: u64,
    ) -> Self {
        Self::new(
            member_id,
            action_id,
            ExtensionMessageKind::FillForm {
                target,
                fields,
                wait_for_human_confirm: true,
                timeout_seconds,
            },
        )
    }

    pub fn ping(member_id: &str) -> Self {
        Self::new(member_id, "", ExtensionMessageKind::Ping)
    }

    pub fn serialize(&self) -> Result<String, TrackBError> {
        serde_json::to_string(self).map_err(|e| TrackBError::Serialization(e.to_string()))
    }

    pub fn deserialize(s: &str) -> Result<Self, TrackBError> {
        serde_json::from_str(s).map_err(|e| TrackBError::Serialization(e.to_string()))
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_form_roundtrip() {
        let msg = BrowserExtensionMessage::fill_form(
            "m_001",
            "act_123",
            FillTarget::SendMessagePage,
            serde_json::json!({"to": "alice", "body": "Hi"}),
            60,
        );
        let s = msg.serialize().unwrap();
        let back = BrowserExtensionMessage::deserialize(&s).unwrap();
        assert_eq!(back.member_id, "m_001");
        assert_eq!(back.action_id, "act_123");
        match back.message {
            ExtensionMessageKind::FillForm { target, .. } => {
                assert_eq!(target, FillTarget::SendMessagePage);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn ping_message() {
        let msg = BrowserExtensionMessage::ping("m_001");
        match msg.message {
            ExtensionMessageKind::Ping => {}
            _ => panic!("wrong variant"),
        }
    }
}
