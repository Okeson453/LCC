"""Event envelope — wraps every async fan-out payload.

Mirrors `crates/events/src/envelope.rs` on the Rust side. Both engines
agree on this shape via `schemas/events/*.schema.json` (JSON-Schema).
"""

from __future__ import annotations

import json
import uuid
from datetime import UTC, datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class Topic(str, Enum):
    MEMBER_CREATED = "member.created"
    MEMBER_DEACTIVATED = "member.deactivated"
    OAUTH_TOKEN_REFRESH_FAILED = "oauth.token_refresh_failed"

    PROFILE_SNAPSHOT_CREATED = "profile.snapshot.created"
    PROFILE_AUDIT_COMPLETED = "profile.audit.completed"

    KB_RECORD_CREATED = "kb.record.created"
    KB_RECORD_UPDATED = "kb.record.updated"
    KB_RECORD_DELETED = "kb.record.deleted"
    VOICE_SAMPLE_ADDED = "voice.sample.added"

    CONTENT_ITEM_STATE_CHANGED = "content.item.state_changed"
    CONTENT_ITEM_APPROVED = "content.item.approved"

    ENGAGEMENT_INBOUND_RECEIVED = "engagement.inbound.received"
    ENGAGEMENT_REPLY_DRAFTED = "engagement.reply_drafted"
    SEQUENCE_REPLY_DETECTED = "sequence.reply_detected"
    SEQUENCE_STEP_DUE = "sequence.step.due"
    SEQUENCE_STATE_CHANGED = "sequence.state_changed"
    SEQUENCE_STEP_SENT = "sequence.step.sent"

    OPPORTUNITY_DISCOVERED = "opportunity.discovered"
    OPPORTUNITY_QUALIFIED = "opportunity.qualified"
    OPPORTUNITY_FUNNEL_CHANGED = "opportunity.funnel_changed"

    APPROVAL_DECIDED = "approval.decided"
    APPROVAL_EXPIRED = "approval.expired"

    COMPLIANCE_CONFIG_ACTIVATED = "compliance.config_activated"
    COMPLIANCE_RESTRICTION_DETECTED = "compliance.restriction_detected"
    COMPLIANCE_RESTRICTION_CLEARED = "compliance.restriction_cleared"

    AUDIT_EVENT = "audit.event"


class EventHeader(BaseModel):
    model_config = ConfigDict(extra="forbid")

    event_id: str
    topic: Topic
    trace_id: str
    producer_service: str
    occurred_at: datetime
    member_id: str | None = None
    idempotency_key: str
    schema_version: int = 1


# Concrete payload types — kept as TypedDict-like dicts to match the JSON Schema.

class AuditEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    actor: str
    action: str
    resource_type: str
    resource_id: str | None = None
    outcome: str
    reason: str | None = None
    checksum_sha256: str
    metadata: dict[str, Any] = Field(default_factory=dict)


class MemberCreatedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    member_id: str
    linkedin_id: str
    created_at: datetime


class KbRecordCreatedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    record_id: str
    member_id: str
    category: str


class ContentItemApprovedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    item_id: str
    member_id: str
    scheduled_at: datetime | None = None


class ComplianceConfigActivatedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    version_id: str
    version: str
    activated_at: datetime
    activated_by: str
    two_reviewer_signed_by: list[str]


class ComplianceRestrictionDetectedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    member_id: str
    reason: str
    signal_kind: str


class SequenceReplyDetectedEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    sequence_id: str
    member_id: str
    contact_id: str
    inbound_message_id: str


class EventEnvelope(BaseModel):
    """Top-level envelope."""

    model_config = ConfigDict(extra="forbid")

    header: EventHeader
    payload: dict[str, Any]

    @classmethod
    def new(
        cls,
        topic: Topic,
        producer_service: str,
        trace_id: str,
        payload: dict[str, Any],
        member_id: str | None = None,
    ) -> EventEnvelope:
        event_id = str(uuid.uuid4())
        return cls(
            header=EventHeader(
                event_id=event_id,
                topic=topic,
                trace_id=trace_id,
                producer_service=producer_service,
                occurred_at=datetime.now(UTC),
                member_id=member_id,
                idempotency_key=f"{topic.value}:{event_id}",
                schema_version=1,
            ),
            payload=payload,
        )

    def to_json(self) -> str:
        return self.model_dump_json()

    @classmethod
    def from_json(cls, s: str) -> EventEnvelope:
        return cls.model_validate_json(s)


# Back-compat: a single `EventPayload` enum-like wrapper for the publisher/consumer.
EventPayload = dict[str, Any]
