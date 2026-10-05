//! Tests for the event envelope.
//!
//! Written against the real `lcc_events` surface: `Envelope::new` takes a
//! typed `EventPayload` and the member id is attached with `with_member`;
//! serialisation is plain serde. The previous version passed a raw
//! `serde_json::Value` plus a 5th positional member argument and called
//! `to_json` / `from_json`, none of which exist.
// Integration tests assert on real return values; `unwrap`/`expect` on a
// failing assertion is the point, so the production deny does not apply.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use lcc_events::envelope::{Envelope, EventPayload, MemberCreatedEvent};
use lcc_events::topics::Topic;

fn member_created_payload() -> EventPayload {
    EventPayload::MemberCreated(MemberCreatedEvent {
        member_id: "m-1".into(),
        linkedin_id: "li-1".into(),
        created_at: Utc::now(),
    })
}

#[test]
fn envelope_round_trip_json() {
    let env = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "trace-123",
        member_created_payload(),
    )
    .with_member("m-1");

    let s = serde_json::to_string(&env).expect("serialize");
    let parsed: Envelope = serde_json::from_str(&s).expect("deserialize round-trip");

    assert_eq!(parsed.header.topic, Topic::MemberCreated);
    assert_eq!(parsed.header.producer_service, "identity-svc");
    assert_eq!(parsed.header.trace_id, "trace-123");
    assert_eq!(parsed.header.member_id.as_deref(), Some("m-1"));
    // The header is flattened, so member_id sits beside topic, not under it.
    assert_eq!(parsed.header.member_id.as_deref(), Some("m-1"));
    match &parsed.payload {
        EventPayload::MemberCreated(p) => assert_eq!(p.member_id, "m-1"),
        other => panic!("payload changed variant: {other:?}"),
    }
}

#[test]
fn idempotency_key_is_namespaced_by_topic() {
    let env = Envelope::new(
        Topic::ContentItemApproved,
        "content-svc",
        "trace-1",
        EventPayload::ContentItemApproved(lcc_events::envelope::ContentItemApprovedEvent {
            item_id: "i-1".into(),
            member_id: "m-1".into(),
            scheduled_at: None,
        }),
    );
    let topic = env.header.topic.as_str();
    assert!(
        env.header.idempotency_key.starts_with(&format!("{topic}:")),
        "idempotency key {:?} is not namespaced by topic {topic}",
        env.header.idempotency_key
    );
}

#[test]
fn idempotency_keys_are_unique_per_event() {
    let a = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t",
        member_created_payload(),
    );
    let b = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t",
        member_created_payload(),
    );
    assert_ne!(
        a.header.idempotency_key, b.header.idempotency_key,
        "two events shared an idempotency key"
    );
    assert_ne!(a.header.event_id, b.header.event_id);
}

#[test]
fn schema_version_is_one() {
    let env = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t-1",
        member_created_payload(),
    );
    assert_eq!(env.header.schema_version, 1);
}

#[test]
fn member_id_is_absent_for_system_events() {
    let env = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t-1",
        member_created_payload(),
    );
    assert!(env.header.member_id.is_none());
}

#[test]
fn event_id_is_a_fresh_uuid_per_event() {
    let a = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t-1",
        member_created_payload(),
    );
    let b = Envelope::new(
        Topic::MemberCreated,
        "identity-svc",
        "t-1",
        member_created_payload(),
    );
    // Event ids are v7 uuids, not the trace id, so consumers can dedupe on them.
    assert_ne!(a.header.event_id, b.header.event_id);
}
