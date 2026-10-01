//! Briefing kind enum mirrored from lcc.briefing_kind PG enum.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BriefingKind {
    Morning,
    Midday,
    Evening,
    Custom,
}

impl BriefingKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Morning => "morning",
            Self::Midday => "midday",
            Self::Evening => "evening",
            Self::Custom => "custom",
        }
    }
}

impl Default for BriefingKind {
    fn default() -> Self {
        Self::Morning
    }
}
