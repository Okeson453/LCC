//! Briefing kind enum mirrored from lcc.briefing_kind PG enum.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BriefingKind {
    // The documented default, so it is marked rather than spelled out in a
    // hand-written `impl Default`.
    #[default]
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
