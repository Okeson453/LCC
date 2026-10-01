"""event-bus — Async event bus client for Python Intelligence services.

Phase 1-2: Redis Streams. Phase 3+: Kafka (rdkafka).
"""

from event_bus.envelope import (
    Envelope,
    EventHeader,
    EventPayload,
    AuditEvent,
    MemberCreatedEvent,
    KbRecordCreatedEvent,
    ContentItemApprovedEvent,
    ComplianceConfigActivatedEvent,
    ComplianceRestrictionDetectedEvent,
    SequenceReplyDetectedEvent,
)
from event_bus.publisher import Publisher, PublisherError
from event_bus.consumer import Consumer, ConsumerError

__all__ = [
    "Envelope",
    "EventHeader",
    "EventPayload",
    "AuditEvent",
    "MemberCreatedEvent",
    "KbRecordCreatedEvent",
    "ContentItemApprovedEvent",
    "ComplianceConfigActivatedEvent",
    "ComplianceRestrictionDetectedEvent",
    "SequenceReplyDetectedEvent",
    "Publisher",
    "PublisherError",
    "Consumer",
    "ConsumerError",
]
