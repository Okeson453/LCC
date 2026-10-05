//! KB domain types — knowledge records with vector embeddings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbRecord {
    pub id: Uuid,
    pub member_id: Uuid,
    pub kind: KbKind,
    pub title: String,
    pub body: String,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub embedding_id: Option<String>,
    pub embedding_status: EmbeddingStatus,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KbKind {
    Experience,
    Skill,
    Project,
    Principle,
    VoiceStyle,
    Industry,
    Persona,
}

impl KbKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Experience => "experience",
            Self::Skill => "skill",
            Self::Project => "project",
            Self::Principle => "principle",
            Self::VoiceStyle => "voice_style",
            Self::Industry => "industry",
            Self::Persona => "persona",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingStatus {
    Pending,
    Embedded,
    Failed,
    Stale,
}

impl EmbeddingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Embedded => "embedded",
            Self::Failed => "failed",
            Self::Stale => "stale",
        }
    }
}

// Tests assert on real return values; `unwrap`/`expect` on a failing
// assertion is the point, so the production deny does not apply here.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn kind_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&KbKind::VoiceStyle).unwrap(),
            "\"voice_style\""
        );
        assert_eq!(
            serde_json::to_string(&KbKind::Industry).unwrap(),
            "\"industry\""
        );
    }

    #[test]
    fn embedding_status_enum() {
        assert_eq!(EmbeddingStatus::Pending.as_str(), "pending");
        assert_eq!(EmbeddingStatus::Stale.as_str(), "stale");
    }
}
