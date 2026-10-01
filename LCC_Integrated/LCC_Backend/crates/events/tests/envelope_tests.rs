//! Tests for the event envelope.

use lcc_events::envelope::Envelope;
use lcc_events::topics::Topic;

#[test]
fn envelope_round_trip_json() {
    let env = Envelope::new(
        Topic::MemberCreated,
        "identity-svc".into(),
        "trace-123".into(),
        serde_json::json!({"member_id": "m-1"}),
        Some("m-1".into()),
    );
    let s = env.to_json();
    let parsed = Envelope::from_json(&s).expect("parse round-trip");
    assert_eq!(parsed.header.topic, Topic::MemberCreated);
    assert_eq!(parsed.payload["member_id"], "m-1");
}

#[test]
fn idempotency_key_format() {
    let env = Envelope::new(
        Topic::ContentItemApproved,
        "content-svc".into(),
        "trace-1".into(),
        serde_json::json!({"item_id": "i-1"}),
        None,
    );
    assert!(env.header.idempotency_key.starts_with("content.item.approved:"));
}

#[test]
fn schema_version_is_one() {
    let env = Envelope::new(
        Topic::AuditEvent,
        "audit-svc".into(),
        "t-1".into(),
        serde_json::json!({}),
        None,
    );
    assert_eq!(env.header.schema_version, 1);
}
