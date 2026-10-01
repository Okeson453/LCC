"""Event-bus envelope tests."""

import json
from datetime import UTC, datetime

import pytest

from event_bus.envelope import EventEnvelope, EventHeader, Topic


@pytest.mark.unit
class TestEventEnvelope:
    def test_create_envelope(self):
        env = EventEnvelope.new(
            topic=Topic.MEMBER_CREATED,
            producer_service="identity-svc",
            trace_id="trace-1",
            payload={"member_id": "m-1"},
            member_id="m-1",
        )
        assert env.header.topic == Topic.MEMBER_CREATED
        assert env.header.producer_service == "identity-svc"
        assert env.payload["member_id"] == "m-1"

    def test_envelope_round_trip_json(self):
        env = EventEnvelope.new(
            topic=Topic.KB_RECORD_CREATED,
            producer_service="kb-intel",
            trace_id="trace-2",
            payload={"record_id": "r-1"},
        )
        s = env.to_json()
        parsed = EventEnvelope.from_json(s)
        assert parsed.header.topic == Topic.KB_RECORD_CREATED
        assert parsed.payload["record_id"] == "r-1"

    def test_envelope_idempotency_key_format(self):
        env = EventEnvelope.new(
            topic=Topic.CONTENT_ITEM_APPROVED,
            producer_service="content-svc",
            trace_id="t-3",
            payload={"item_id": "i-1"},
        )
        # idempotency_key includes topic + event_id
        assert env.header.idempotency_key.startswith("content.item.approved:")
        assert env.header.event_id in env.header.idempotency_key

    def test_envelope_extra_forbid(self):
        env_dict = {
            "header": {
                "event_id": "id",
                "topic": "member.created",
                "trace_id": "t",
                "producer_service": "p",
                "occurred_at": datetime.now(UTC).isoformat(),
                "idempotency_key": "k",
                "schema_version": 1,
            },
            "payload": {"x": 1},
            "extra_field": "should_fail",
        }
        with pytest.raises(Exception):
            EventEnvelope.model_validate(env_dict)


@pytest.mark.unit
class TestTopic:
    def test_topic_values_unique(self):
        values = [t.value for t in Topic]
        assert len(values) == len(set(values))

    def test_member_created_topic(self):
        assert Topic.MEMBER_CREATED.value == "member.created"

    def test_compliance_topics(self):
        assert Topic.COMPLIANCE_CONFIG_ACTIVATED.value == "compliance.config_activated"
        assert Topic.COMPLIANCE_RESTRICTION_DETECTED.value == "compliance.restriction_detected"
