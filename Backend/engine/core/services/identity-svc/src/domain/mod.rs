//! Identity domain types.
//!
//! Phase C implementation per contract_audit/openapi/lcc-api-canonical.yaml.
//! Replaces the previous generic Entity/EntityState scaffold with the real
//! domain model required by the canonical contract.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Goal mode for the account, per design §3.1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum GoalMode {
    JobHunting,
    ClientAcquisition,
    // The documented default, so it is marked rather than spelled out in a
    // hand-written `impl Default`.
    #[default]
    Hybrid,
}

impl GoalMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::JobHunting => "job_hunting",
            Self::ClientAcquisition => "client_acquisition",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Assistant,
    Reviewer,
    Admin,
    Auditor,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Assistant => "assistant",
            Self::Reviewer => "reviewer",
            Self::Admin => "admin",
            Self::Auditor => "auditor",
        }
    }

    /// Parse from the JWT `role` claim string.
    ///
    /// Named `parse_role` rather than `from_str` so it does not read as an
    /// implementation of `std::str::FromStr` — which it is not: it takes
    /// `&str` and cannot fail, falling back to `Owner`.
    pub fn parse_role(s: &str) -> Self {
        match s {
            "owner" => Self::Owner,
            "assistant" => Self::Assistant,
            "reviewer" => Self::Reviewer,
            "admin" => Self::Admin,
            "auditor" => Self::Auditor,
            _ => Self::Owner, // Default to owner for unknown strings.
        }
    }
}

/// Member account — the canonical Member entity.
///
/// Mirrors `lcc.members` plus optional extended fields from settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: Uuid,
    pub linkedin_id: String,
    pub email: Option<String>,
    pub display_name: String,
    pub role: MemberRole,
    pub is_active: bool,
    pub timezone: String,
    pub locale: String,
    pub active_goal_mode: GoalMode,
    pub is_restricted: bool,
    pub restricted_since: Option<DateTime<Utc>>,
    pub restricted_reason: Option<String>,
    pub warmup_started_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Member settings — sub-resource.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberSettings {
    pub timezone: String,
    pub active_goal_mode: GoalMode,
    /// Optional per-member cap overrides. Each value, when present, overrides
    /// the base cap from the active compliance config. None means "use base".
    pub cap_overrides: Option<CapOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_per_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dm_per_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_per_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_per_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_per_day: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberSettingsUpdate {
    pub timezone: Option<String>,
    pub active_goal_mode: Option<GoalMode>,
    pub cap_overrides: Option<CapOverrides>,
}

/// OAuth token row from `lcc.oauth_tokens`.
#[derive(Debug, Clone)]
pub struct OAuthToken {
    pub member_id: Uuid,
    pub provider: String,
    pub access_token_encrypted: Vec<u8>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub expires_at: DateTime<Utc>,
    pub scope: Option<String>,
}

/// Returned by POST /auth/refresh.
#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str, // always "Bearer"
    pub expires_in: i64,          // seconds
}

/// LinkedIn OAuth handshake — returned by /auth/linkedin/start.
#[derive(Debug, Clone, Serialize)]
pub struct LinkedInStart {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String, // PKCE; the client should keep this to verify on callback
    pub scopes: Vec<String>,
}

/// Result of exchanging the OAuth code.
#[derive(Debug, Clone)]
pub struct LinkedInExchange {
    pub member_id: Uuid,
    pub access_token_encrypted: Vec<u8>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub expires_at: DateTime<Utc>,
    pub scope: Option<String>,
}
