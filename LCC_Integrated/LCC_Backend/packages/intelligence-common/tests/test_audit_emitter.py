"""Tests for the audit_emitter helpers."""

import hashlib
import json
import time
from unittest.mock import AsyncMock

import pytest

from intelligence_common.audit_emitter import (
    LLMCallAuditEvent,
    compute_completion_hash,
    compute_prompt_hash,
)


@pytest.mark.unit
class TestHashing:
    def test_compute_prompt_hash_stable(self):
        text = "write a draft about kubernetes."
        h1 = compute_prompt_hash(text)
        h2 = compute_prompt_hash(text)
        assert h1 == h2
        assert len(h1) == 64  # sha256 hex

    def test_compute_completion_hash_stable(self):
        text = "shipped today. latency down 30%."
        h1 = compute_completion_hash(text)
        h2 = compute_completion_hash(text)
        assert h1 == h2

    def test_different_text_different_hash(self):
        h1 = compute_prompt_hash("a")
        h2 = compute_prompt_hash("b")
        assert h1 != h2


@pytest.mark.unit
class TestLLMCallAuditEvent:
    def test_event_serialization_round_trip(self):
        event = LLMCallAuditEvent(
            actor="system:ai-worker",
            action="llm.draft_content",
            resource_id="item-123",
            model_id="gpt-4o-2024-08-06",
            prompt_hash="abc",
            completion_hash="def",
            tokens_in=100,
            tokens_out=200,
            cost_usd=0.005,
            latency_ms=1500,
            member_id="m-1",
            trace_id="trace-1",
            idempotency_key="key-1",
            model_tier="premium",
            fallback_used=False,
            prompt_kind="draft",
        )
        s = event.model_dump_json()
        data = json.loads(s)
        assert data["action"] == "llm.draft_content"
        assert data["tokens_in"] == 100
        assert data["model_tier"] == "premium"

    def test_event_required_fields(self):
        with pytest.raises(Exception):
            LLMCallAuditEvent(
                actor="system:ai-worker",
                action="llm.draft_content",
                resource_id="item-123",
                model_id="gpt-4o-2024-08-06",
                prompt_hash="abc",
                completion_hash="def",
                tokens_in=100,
                tokens_out=200,
                cost_usd=0.005,
                latency_ms=1500,
                # missing member_id, trace_id, idempotency_key, model_tier, fallback_used, prompt_kind
            )
