//! Content state machine tests.
//!
//! The previous version of this file had two tests that both asserted `true`,
//! so the `ContentState::can_transition_to` table was entirely untested: a
//! regression that allowed `Idea -> Published`, or blocked the
//! `PublishFailed -> Drafted` recovery path, would not have been caught.

use lcc_content_svc::domain::ContentState;
use ContentState::*;

/// The happy path runs end to end without a rejection or a failure.
#[test]
fn draft_to_published_walks_the_whole_chain() {
    let chain = [
        (Idea, Drafted),
        (Drafted, InReview),
        (InReview, Approved),
        (Approved, Scheduled),
        (Scheduled, Published),
    ];
    for (from, to) in chain {
        assert!(
            from.can_transition_to(to),
            "{:?} -> {:?} should be allowed",
            from,
            to
        );
    }
}

/// Publishing goes through `Scheduled` — the content FSM has no direct
/// `Approved -> Published` edge.
///
/// Note the asymmetry with the orchestrator's case FSM, which does allow an
/// approved case to publish without being scheduled
/// (`(Approved, PublishStarted) | (Scheduled, PublishStarted)`). Nothing in
/// the content schema or contract says an approved item may skip scheduling, so
/// this pins the current table rather than asserting the two agree.
#[test]
fn approved_must_be_scheduled_before_publishing() {
    assert!(Approved.can_transition_to(Scheduled));
    assert!(!Approved.can_transition_to(Published));
}

/// A reviewer rejecting a draft returns it to `Rejected` from any active
/// state, which is how a member gets their item back for rework.
#[test]
fn rejection_is_allowed_from_every_active_state() {
    for from in [Drafted, InReview, Approved, Scheduled] {
        assert!(from.can_transition_to(Rejected), "{:?} -> Rejected", from);
    }
}

/// A failed publish must be recoverable, otherwise one transient platform
/// error strands the item permanently.
#[test]
fn publish_failure_is_recoverable() {
    assert!(Scheduled.can_transition_to(PublishFailed));
    assert!(PublishFailed.can_transition_to(Scheduled), "re-schedule");
    assert!(PublishFailed.can_transition_to(Drafted), "edit and retry");
}

/// A blocked item can be unblocked back into drafting.
#[test]
fn blocked_items_can_be_released() {
    assert!(Drafted.can_transition_to(Blocked));
    assert!(Blocked.can_transition_to(Drafted));
}

/// Terminal and skipped states are not edges. These are the cases most likely
/// to be introduced by accident when the table is edited.
#[test]
fn skipped_and_terminal_transitions_are_rejected() {
    for (from, to) in [
        (Idea, Published),
        (Approved, Published),
        (Idea, Approved),
        (Drafted, Published),
        (InReview, Published),
        (Published, Drafted),
        (Published, InReview),
        (Rejected, Approved),
        (Blocked, Approved),
        (Blocked, Published),
        (Rejected, Published),
    ] {
        assert!(
            !from.can_transition_to(to),
            "{:?} -> {:?} should be rejected",
            from,
            to
        );
    }
}

/// A self-transition is never meaningful: nothing changed.
#[test]
fn no_state_transitions_to_itself() {
    for s in [
        Idea,
        Drafted,
        InReview,
        Approved,
        Scheduled,
        Published,
        Rejected,
        PublishFailed,
        Blocked,
    ] {
        assert!(!s.can_transition_to(s), "{:?} -> itself", s);
    }
}

/// `as_str` is what lands in the database column and on the wire, so the
/// spelling is part of the contract.
#[test]
fn state_names_match_the_persisted_spelling() {
    assert_eq!(InReview.as_str(), "in_review");
    assert_eq!(PublishFailed.as_str(), "publish_failed");
    assert_eq!(Idea.as_str(), "idea");
    assert_eq!(Published.as_str(), "published");
}
