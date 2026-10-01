//! Integration test for the case FSM.

use lcc_orchestrator::case::{Case, CaseState, TransitionTrigger};
use uuid::Uuid;

#[test]
fn case_intake_to_published() {
    let mut case = Case::new(Uuid::new_v4());
    assert_eq!(case.state, CaseState::Intake);

    case.try_transition(TransitionTrigger::StartDraft).unwrap();
    case.try_transition(TransitionTrigger::DraftComplete).unwrap();
    case.try_transition(TransitionTrigger::Approve).unwrap();
    case.try_transition(TransitionTrigger::PublishStarted).unwrap();
    case.try_transition(TransitionTrigger::PublishComplete).unwrap();
    assert_eq!(case.state, CaseState::Published);
}

#[test]
fn case_reject_loop() {
    let mut case = Case::new(Uuid::new_v4());
    case.try_transition(TransitionTrigger::StartDraft).unwrap();
    case.try_transition(TransitionTrigger::DraftComplete).unwrap();
    case.try_transition(TransitionTrigger::Reject).unwrap();
    assert_eq!(case.state, CaseState::Drafting);

    // Second loop.
    case.try_transition(TransitionTrigger::DraftComplete).unwrap();
    case.try_transition(TransitionTrigger::Approve).unwrap();
    case.try_transition(TransitionTrigger::Schedule).unwrap();
    case.try_transition(TransitionTrigger::PublishStarted).unwrap();
    case.try_transition(TransitionTrigger::PublishFailed).unwrap();
    assert_eq!(case.state, CaseState::Failed);
}

#[test]
fn case_illegal_trigger() {
    let mut case = Case::new(Uuid::new_v4());
    let result = case.try_transition(TransitionTrigger::PublishStarted);
    assert!(result.is_err());
}
