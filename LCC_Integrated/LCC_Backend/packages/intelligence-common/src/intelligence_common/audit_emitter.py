"""Audit emission helper for the Intelligence Engine.

Every LLM call must emit an audit event with prompt_hash, completion_hash,
model_id, tokens, cost_usd, latency_ms.
"""

from __future__ import annotations

import hashlib
import time
import uuid
from contextlib import asynccontextmanager
from dataclasses import dataclass, field
from typing import AsyncIterator

import httpx
import structlog

log = structlog.get_logger(__name__)


def compute_prompt_hash(prompt: str) -> str:
    """Stable SHA-256 of the prompt text."""
    return hashlib.sha256(prompt.encode("utf-8")).hexdigest()


def compute_completion_hash(completion: str) -> str:
    """Stable SHA-256 of the completion text."""
    return hashlib.sha256(completion.encode("utf-8")).hexdigest()


@dataclass
class LLMCallAuditEvent:
    """Schema for an LLM-call audit event."""

    actor: str
    action: str
    resource_id: str
    model_id: str
    prompt_hash: str
    completion_hash: str
    tokens_in: int
    tokens_out: int
    cost_usd: float
    latency_ms: int
    member_id: str | None
    trace_id: str | None
    idempotency_key: str
    model_tier: str
    fallback_used: bool
    prompt_kind: str
    metadata: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "actor": self.actor,
            "action": self.action,
            "resource_id": self.resource_id,
            "model_id": self.model_id,
            "prompt_hash": self.prompt_hash,
            "completion_hash": self.completion_hash,
            "tokens_in": self.tokens_in,
            "tokens_out": self.tokens_out,
            "cost_usd": self.cost_usd,
            "latency_ms": self.latency_ms,
            "member_id": self.member_id,
            "trace_id": self.trace_id,
            "idempotency_key": self.idempotency_key,
            "model_tier": self.model_tier,
            "fallback_used": self.fallback_used,
            "prompt_kind": self.prompt_kind,
            "metadata": self.metadata,
        }


class LlmCallTimer:
    """Context manager that records wall-clock latency in milliseconds."""

    def __init__(self) -> None:
        self.start: float | None = None
        self.latency_ms: int = 0

    def __enter__(self) -> "LlmCallTimer":
        self.start = time.monotonic()
        return self

    def __exit__(self, *args: object) -> None:
        if self.start is not None:
            self.latency_ms = int((time.monotonic() - self.start) * 1000)


class AuditEmitter:
    """HTTP-based audit emitter for LLM-call events.

    The Intelligence Engine writes to the audit-svc over HTTP rather than
    directly to Postgres (so the audit-svc owns the audit chain integrity).
    """

    def __init__(self, audit_svc_url: str, timeout_seconds: float = 5.0) -> None:
        self._url = audit_svc_url
        self._http: httpx.AsyncClient | None = None

    async def __aenter__(self) -> "AuditEmitter":
        self._http = httpx.AsyncClient(timeout=self._timeout_seconds)
        return self

    @property
    def _timeout_seconds(self) -> float:
        return 5.0

    async def __aexit__(self, *args: object) -> None:
        if self._http:
            await self._http.aclose()

    async def emit(self, event: LLMCallAuditEvent) -> None:
        """POST an audit event to the audit-svc. Best-effort: failures logged."""
        if self._http is None:
            raise RuntimeError("AuditEmitter must be used as async context manager")
        body = {
            "actor": event.actor,
            "action": event.action,
            "resource_type": "llm_call",
            "resource_id": event.resource_id,
            "outcome": "ok",
            "reason": None,
            "metadata": event.to_dict(),
        }
        try:
            resp = await self._http.post(
                f"{self._url}/v1/audit/events",
                json=body,
                headers={"Idempotency-Key": event.idempotency_key},
            )
            if resp.status_code >= 400:
                log.warning(
                    "audit_emit_failed",
                    status=resp.status_code,
                    body=resp.text[:200],
                )
        except httpx.HTTPError as e:
            log.warning("audit_emit_error", error=str(e))


def make_idempotency_key(action: str, resource_id: str) -> str:
    """Build an idempotency key for an audit event."""
    return f"{action}:{resource_id}:{uuid.uuid4()}"
