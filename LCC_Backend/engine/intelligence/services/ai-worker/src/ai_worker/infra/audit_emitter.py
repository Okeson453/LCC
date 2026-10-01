"""Audit emission for the ai-worker.

Wraps `intelligence_common.audit_emitter` for HTTP-callable use.
"""

from __future__ import annotations

import os
import time

from intelligence_common.audit_emitter import (
    AuditEmitter,
    LLMCallAuditEvent,
    LlmCallTimer,
    compute_completion_hash,
    compute_prompt_hash,
)


async def emit_audit_event(
    *,
    member_id: str | None,
    action: str,
    resource_id: str | None,
    model_id: str | None,
    tokens_in: int,
    tokens_out: int,
    cost_usd: float,
    latency_ms: int,
    prompt_kind: str,
    trace_id: str | None,
    fallback_used: bool = False,
    prompt_text: str = "",
    completion_text: str = "",
) -> None:
    """Emit an audit event for an LLM call."""
    audit_svc_url = os.environ.get("AUDIT_SVC_URL", "http://audit-svc:8080")
    async with AuditEmitter(audit_svc_url) as emitter:
        event = LLMCallAuditEvent(
            actor=f"system:ai-worker",
            action=action,
            resource_id=resource_id or "",
            model_id=model_id or "rule_based",
            prompt_hash=compute_prompt_hash(prompt_text) if prompt_text else "n/a",
            completion_hash=compute_completion_hash(completion_text) if completion_text else "n/a",
            tokens_in=tokens_in,
            tokens_out=tokens_out,
            cost_usd=cost_usd,
            latency_ms=latency_ms,
            member_id=member_id,
            trace_id=trace_id,
            idempotency_key=f"ai-worker:{action}:{resource_id or 'n/a'}:{int(time.time() * 1000)}",
            model_tier="rule_based" if fallback_used else "premium",
            fallback_used=fallback_used,
            prompt_kind=prompt_kind,
        )
        await emitter.emit(event)


__all__ = ["emit_audit_event", "LlmCallTimer"]
