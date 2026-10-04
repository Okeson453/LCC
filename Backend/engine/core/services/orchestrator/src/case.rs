//! Case lifecycle state machine.
//!
//! A `Case` is one end-to-end content production run: it is taken from
//! `intake`, drafted, reviewed, approved, scheduled, published, and either
//! lands in `published` or is sent back for another pass or ends `failed`.
//!
//! Transitions are explicit and total: `try_transition` returns an error for
//! any trigger that is not legal from the current state, so an out-of-order
//! event can never silently advance a case.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// The states a case moves through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    /// Accepted, not yet drafted.
    Intake,
    /// Being written; a `reject` during review returns here.
    Drafting,
    /// Draft complete, waiting on a reviewer.
    InReview,
    /// Approved and ready to schedule or publish.
    Approved,
    /// Queued for publication.
    Scheduled,
    /// Handed to the publication path.
    Publishing,
    /// Terminal success.
    Published,
    /// Terminal failure — the publish attempt did not complete.
    Failed,
}

impl CaseState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Intake => "intake",
            Self::Drafting => "drafting",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Scheduled => "scheduled",
            Self::Publishing => "publishing",
            Self::Published => "published",
            Self::Failed => "failed",
        }
    }

    /// True once the case can no longer change.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Published | Self::Failed)
    }
}

/// An event that asks the case to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionTrigger {
    StartDraft,
    DraftComplete,
    Approve,
    Reject,
    Schedule,
    PublishStarted,
    PublishComplete,
    PublishFailed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CaseTransitionError {
    #[error("trigger {trigger:?} is not legal from state {state:?}")]
    Illegal { state: CaseState, trigger: TransitionTrigger },
    #[error("case is {state:?}, which is terminal")]
    Terminal { state: CaseState },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: Uuid,
    pub state: CaseState,
}

impl Case {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: CaseState::Intake,
        }
    }

    /// The single source of truth for legal transitions.
    fn next_state(
        state: CaseState,
        trigger: TransitionTrigger,
    ) -> Option<CaseState> {
        use CaseState::*;
        use TransitionTrigger::*;
        match (state, trigger) {
            (Intake, StartDraft) => Some(Drafting),
            (Drafting, DraftComplete) => Some(InReview),
            // A rejected review sends the case back for another pass rather
            // than failing it, so the author can address the feedback. This
            // must land on `Drafting`: staying in `InReview` would leave the
            // case queued for a review that already came back negative, and
            // `try_transition` would report success while nothing moved.
            (InReview, Reject) => Some(Drafting),
            (InReview, Approve) => Some(Approved),
            (Approved, Schedule) => Some(Scheduled),
            // An approved case may publish immediately or after scheduling.
            (Approved, PublishStarted) | (Scheduled, PublishStarted) => Some(Publishing),
            (Publishing, PublishComplete) => Some(Published),
            (Publishing, PublishFailed) => Some(Failed),
            _ => None,
        }
    }

    /// Apply `trigger`, or leave the case untouched and explain why not.
    pub fn try_transition(&mut self, trigger: TransitionTrigger) -> Result<(), CaseTransitionError> {
        if self.state.is_terminal() {
            return Err(CaseTransitionError::Terminal { state: self.state });
        }
        match Self::next_state(self.state, trigger) {
            Some(next) => {
                self.state = next;
                Ok(())
            }
            None => Err(CaseTransitionError::Illegal {
                state: self.state,
                trigger,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_names_round_trip() {
        for s in [
            CaseState::Intake,
            CaseState::Drafting,
            CaseState::InReview,
            CaseState::Approved,
            CaseState::Scheduled,
            CaseState::Publishing,
            CaseState::Published,
            CaseState::Failed,
        ] {
            let json = serde_json::to_string(&s).unwrap_or_default();
            let back: CaseState = serde_json::from_str(&json).unwrap_or(CaseState::Intake);
            assert_eq!(s, back);
        }
    }

    #[test]
    fn terminal_states_reject_everything() {
        let mut c = Case::new(Uuid::new_v4());
        c.state = CaseState::Published;
        assert!(matches!(
            c.try_transition(TransitionTrigger::DraftComplete),
            Err(CaseTransitionError::Terminal { .. })
        ));
        assert_eq!(c.state, CaseState::Published);
    }

    #[test]
    fn illegal_transition_leaves_state_untouched() {
        let mut c = Case::new(Uuid::new_v4());
        assert!(c.try_transition(TransitionTrigger::Approve).is_err());
        assert_eq!(c.state, CaseState::Intake);
    }
}
