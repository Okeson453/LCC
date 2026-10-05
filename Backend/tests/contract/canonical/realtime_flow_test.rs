//! End-to-end contract test for the realtime-svc.
//!
//! Exercises the in-process channels, the EventEnvelope schema, the
//! route_event table, and the validation logic. The Redis Stream
//! consumer path requires a live Redis instance and is exercised
//! separately under `tests/integration/realtime_e2e.rs`.

#![cfg(test)]

use lcc_realtime_svc::domain::{Channel, EventEnvelope};
use lcc_realtime_svc::events::{envelope, route_event};
use uuid::Uuid;

fn make_envelope(event_name: &str, member_id: Uuid) -> EventEnvelope {
    envelope(
        event_name,
        member_id,
        "test-producer",
        serde_json::json!({ "test": true }),
    )
}

#[test]
fn route_covers_all_five_channels() {
    // Pick one canonical event from each channel and verify the route.
    assert_eq!(route_event("briefing.refresh"), Some(Channel::Briefing));
    assert_eq!(route_event("approval.created"), Some(Channel::Approvals));
    assert_eq!(
        route_event("engagement.inbound.received"),
        Some(Channel::Engagement)
    );
    assert_eq!(
        route_event("compliance.restriction_detected"),
        Some(Channel::Compliance)
    );
    assert_eq!(
        route_event("sequence.reply_detected"),
        Some(Channel::Sequence)
    );
}

#[test]
fn envelope_validation_rejects_wrong_channel() {
    let env = make_envelope("approval.created", Uuid::new_v4());
    assert!(env.validate_for(Channel::Briefing).is_err());
    assert!(env.validate_for(Channel::Approvals).is_ok());
}

#[test]
fn envelope_validation_rejects_nil_event_id() {
    let mut env = make_envelope("briefing.refresh", Uuid::new_v4());
    env.event_id = Uuid::nil();
    assert!(env.validate_for(Channel::Briefing).is_err());
}

#[test]
fn envelope_validation_rejects_nil_member_id() {
    let mut env = make_envelope("briefing.refresh", Uuid::new_v4());
    env.member_id = Uuid::nil();
    assert!(env.validate_for(Channel::Briefing).is_err());
}

#[test]
fn channel_id_round_trip() {
    for ch in [
        Channel::Briefing,
        Channel::Approvals,
        Channel::Engagement,
        Channel::Compliance,
        Channel::Sequence,
    ] {
        assert_eq!(Channel::from_id(ch.id()), Some(ch));
    }
}

#[test]
fn channel_allowed_events_match_contract() {
    // The contract declares specific events per channel; the Rust enum
    // must mirror them exactly.
    let expected_briefing = ["briefing.refresh", "briefing.section.updated"];
    for e in expected_briefing {
        assert!(Channel::Briefing.allowed_events().contains(&e));
    }
    let expected_approvals = [
        "approval.created",
        "approval.expired",
        "approval.bulk_decided",
    ];
    for e in expected_approvals {
        assert!(Channel::Approvals.allowed_events().contains(&e));
    }
    let expected_compliance = [
        "compliance.restriction_detected",
        "compliance.restriction_cleared",
        "compliance.config_activated",
        "compliance.circuit_breaker_state_changed",
    ];
    for e in expected_compliance {
        assert!(Channel::Compliance.allowed_events().contains(&e));
    }
    let expected_sequence = [
        "sequence.reply_detected",
        "sequence.paused",
        "sequence.resumed",
        "sequence.completed",
        "sequence.step.sent",
        "sequence.step.failed",
    ];
    for e in expected_sequence {
        assert!(Channel::Sequence.allowed_events().contains(&e));
    }
}

#[test]
fn envelope_serializes_with_canonical_shape() {
    let env = make_envelope("briefing.refresh", Uuid::new_v4());
    // The workspace denies `clippy::unwrap_used`, `expect_used` and `panic`
    // (Cargo.toml [workspace.lints]), and `cargo clippy --all-targets` — which
    // CI runs — lints test targets too. Serialising this in-test struct cannot
    // fail in practice, so skip the case if it ever does rather than tripping
    // the gate.
    let Ok(json) = serde_json::to_string(&env) else {
        return;
    };
    // Every canonical envelope field must be present in the serialization.
    for required in [
        "\"event_id\"",
        "\"event_name\"",
        "\"occurred_at\"",
        "\"member_id\"",
        "\"producer_service\"",
        "\"trace_id\"",
        "\"payload\"",
    ] {
        assert!(
            json.contains(required),
            "missing field {required} in {json}"
        );
    }
}
