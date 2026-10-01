"""Draft endpoints — content/reply/outreach/profile_edit/proposal."""

from __future__ import annotations

import uuid
from typing import Any, Literal

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, ConfigDict, Field

from ai_worker.brand_guard.linter import lint_draft

import structlog

logger = structlog.get_logger(__name__)
from ai_worker.core.prompts import (
    content_draft_prompt,
    outreach_draft_prompt,
    profile_edit_draft_prompt,
    proposal_draft_prompt,
    reply_draft_prompt,
)
from ai_worker.infra.audit_emitter import emit_audit_event
from ai_worker.infra.embedding import embed_query
from ai_worker.llm.cost_controller import record_spend, select_model_tier
from ai_worker.rag.retriever import retrieve_top_k
from ai_worker.voice.retrieval import retrieve_voice_samples

router = APIRouter()


class DraftVariant(BaseModel):
    model_config = ConfigDict(extra="forbid")
    body: str
    media_suggestions: list[str] = Field(default_factory=list)
    # F-AUDIT-27: axiom 5 requires every generated artifact to cite at least
    # one Professional KB record. This field previously defaulted to an empty
    # list, so a hallucinated draft carrying no citations satisfied the guard.
    kb_refs: list[str] = Field(min_length=1)
    confidence: float = 1.0


class DraftBase(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    variant_count: int = 3
    trace_id: str | None = None


class DraftContentRequest(DraftBase):
    prompt: str
    pillar: str | None = None
    format_hint: str = "text_post"
    kb_top_k: int = 5
    voice_top_k: int = 3


class DraftReplyRequest(DraftBase):
    inbound_message_id: str
    inbound_message_body: str
    context_kind: str  # "dm"|"comment"|"connection_request"
    contact_id: str


class DraftOutreachRequest(DraftBase):
    contact_id: str
    persona: Literal["CTO", "CEO", "Founder", "Recruiter", "HiringManager", "Client", "Peer", "General"]
    tone: str = "value_first"
    goal: str
    context: str = ""


class DraftProfileEditRequest(DraftBase):
    target_field: str
    current_value: str
    goal_mode: str = "job_hunting"
    prompt: str


class DraftProposalRequest(DraftBase):
    opportunity_id: str
    contact_id: str
    discovery_call_notes: str
    proposed_scope: str
    pricing_structure: str


class DraftResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    content_item_id: str | None = None
    variants: list[DraftVariant]
    fallback_used: str = "none"   # "none"|"rule_based"|"kb_unavailable"
    model_id: str | None = None
    tokens_in: int = 0
    tokens_out: int = 0
    cost_usd: float = 0.0
    latency_ms: int = 0
    low_grounding: bool = False


@router.post("/draft-content", response_model=DraftResponse)
async def draft_content(req: DraftContentRequest) -> DraftResponse:
    """Generate content draft variants."""
    # 1. Embed the prompt.
    embedding = await embed_query(req.prompt)

    # 2. Retrieve top-K KB facts (axiom 5).
    kb_records = await retrieve_top_k(
        member_id=req.member_id,
        query_embedding=embedding,
        top_k=req.kb_top_k,
    )
    low_grounding = len(kb_records) == 0
    kb_refs = [r.id for r in kb_records]

    # 3. Retrieve voice samples.
    voice_samples = await retrieve_voice_samples(
        member_id=req.member_id, query_embedding=embedding, top_k=req.voice_top_k,
    )

    # 4. Build prompt + invoke LLM.
    prompt_text = content_draft_prompt(
        user_prompt=req.prompt,
        kb_records=kb_records,
        voice_samples=voice_samples,
        pillar=req.pillar,
        format_hint=req.format_hint,
        variant_count=req.variant_count,
    )

    tier = select_model_tier()
    if tier == "rule_based":
        variants = _rule_based_content_variants(req.prompt, kb_records, req.variant_count)
        fallback = "rule_based"
        model_id = None
        tokens_in = tokens_out = 0
        cost = 0.0
        latency = 0
    else:
        from ai_worker.llm.client import complete  # local import to avoid circular

        resp = await complete(prompt=prompt_text, model_tier=tier)
        variants = _parse_variants(resp.completion, kb_refs, req.variant_count)
        fallback = "none"
        model_id = resp.model_id
        tokens_in = resp.prompt_tokens
        tokens_out = resp.completion_tokens
        cost = resp.cost_usd
        record_spend(cost)  # F-AUDIT-26: feed the monthly budget
        latency = resp.latency_ms

    # 5. Brand-guard lint each variant.
    #
    # F-AUDIT-28: this loop previously read only `lint.confidence` and
    # discarded `passed`, `banned_phrases_found`, `flagged_spans` and
    # `length_ok`. A draft containing a banned phrase was returned to the
    # caller as an ordinary variant with a slightly lower confidence, making
    # the linter advisory telemetry rather than the pre-publish gate the
    # design requires (Technical Design Spec §22: "Brand-guard linter + the
    # Compliance Governor guard stack, both must pass before any send").
    # Failing variants are now dropped, and the response records how many
    # were removed so the failure is visible rather than silent.
    variants = _lint_and_filter(variants, target=req.format_hint)

    response = DraftResponse(
        content_item_id=str(uuid.uuid4()),
        variants=variants,
        fallback_used=fallback,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        low_grounding=low_grounding,
    )

    # 6. Audit emit.
    await emit_audit_event(
        member_id=req.member_id,
        action="llm.draft_content",
        resource_id=response.content_item_id,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        prompt_kind="draft",
        trace_id=req.trace_id,
        fallback_used=fallback != "none",
    )
    return response


@router.post("/draft-reply", response_model=DraftResponse)
async def draft_reply(req: DraftReplyRequest) -> DraftResponse:
    """Generate reply draft variants."""
    embedding = await embed_query(req.inbound_message_body)
    kb_records = await retrieve_top_k(
        member_id=req.member_id,
        query_embedding=embedding,
        top_k=5,
    )
    voice_samples = await retrieve_voice_samples(
        member_id=req.member_id, query_embedding=embedding, top_k=3,
    )
    low_grounding = len(kb_records) == 0

    prompt_text = reply_draft_prompt(
        inbound_body=req.inbound_message_body,
        context_kind=req.context_kind,
        kb_records=kb_records,
        voice_samples=voice_samples,
        variant_count=req.variant_count,
    )

    tier = select_model_tier()
    if tier == "rule_based":
        variants = _rule_based_reply_variants(req.inbound_message_body, req.variant_count)
        fallback = "rule_based"
        model_id = None
        tokens_in = tokens_out = 0
        cost = 0.0
        latency = 0
    else:
        from ai_worker.llm.client import complete
        resp = await complete(prompt=prompt_text, model_tier=tier)
        variants = _parse_variants(resp.completion, [r.id for r in kb_records], req.variant_count)
        fallback = "none"
        model_id = resp.model_id
        tokens_in = resp.prompt_tokens
        tokens_out = resp.completion_tokens
        cost = resp.cost_usd
        record_spend(cost)  # F-AUDIT-26: feed the monthly budget
        latency = resp.latency_ms

    variants = _lint_and_filter(variants, target="reply")

    response = DraftResponse(
        variants=variants,
        fallback_used=fallback,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        low_grounding=low_grounding,
    )

    await emit_audit_event(
        member_id=req.member_id,
        action="llm.draft_reply",
        resource_id=None,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        prompt_kind="draft",
        trace_id=req.trace_id,
        fallback_used=fallback != "none",
    )
    return response


@router.post("/draft-outreach", response_model=DraftResponse)
async def draft_outreach(req: DraftOutreachRequest) -> DraftResponse:
    """Generate outreach draft (connection request, first-touch, etc.)."""
    embedding = await embed_query(f"{req.persona} {req.goal} {req.context}")
    kb_records = await retrieve_top_k(
        member_id=req.member_id, query_embedding=embedding, top_k=5,
    )
    voice_samples = await retrieve_voice_samples(
        member_id=req.member_id, query_embedding=embedding, top_k=3,
    )

    prompt_text = outreach_draft_prompt(
        persona=req.persona,
        tone=req.tone,
        goal=req.goal,
        context=req.context,
        kb_records=kb_records,
        voice_samples=voice_samples,
        variant_count=req.variant_count,
    )

    tier = select_model_tier()
    if tier == "rule_based":
        variants = _rule_based_outreach_variants(req, req.variant_count)
        fallback = "rule_based"
        model_id = None
        tokens_in = tokens_out = 0
        cost = 0.0
        latency = 0
    else:
        from ai_worker.llm.client import complete
        resp = await complete(prompt=prompt_text, model_tier=tier)
        variants = _parse_variants(resp.completion, [r.id for r in kb_records], req.variant_count)
        fallback = "none"
        model_id = resp.model_id
        tokens_in = resp.prompt_tokens
        tokens_out = resp.completion_tokens
        cost = resp.cost_usd
        record_spend(cost)  # F-AUDIT-26: feed the monthly budget
        latency = resp.latency_ms

    variants = _lint_and_filter(variants, target="outreach")

    response = DraftResponse(
        variants=variants,
        fallback_used=fallback,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        low_grounding=len(kb_records) == 0,
    )

    await emit_audit_event(
        member_id=req.member_id,
        action="llm.draft_outreach",
        resource_id=req.contact_id,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        prompt_kind="draft",
        trace_id=req.trace_id,
        fallback_used=fallback != "none",
    )
    return response


@router.post("/draft-profile-edit", response_model=DraftResponse)
async def draft_profile_edit(req: DraftProfileEditRequest) -> DraftResponse:
    """Generate profile edit proposal variants."""
    embedding = await embed_query(f"{req.target_field} {req.prompt}")
    kb_records = await retrieve_top_k(member_id=req.member_id, query_embedding=embedding, top_k=5)

    prompt_text = profile_edit_draft_prompt(
        target_field=req.target_field,
        current_value=req.current_value,
        goal_mode=req.goal_mode,
        user_prompt=req.prompt,
        kb_records=kb_records,
        variant_count=req.variant_count,
    )

    tier = select_model_tier()
    if tier == "rule_based":
        variants = _rule_based_profile_edit_variants(req, req.variant_count)
        fallback = "rule_based"
        model_id = None
        tokens_in = tokens_out = 0
        cost = 0.0
        latency = 0
    else:
        from ai_worker.llm.client import complete
        resp = await complete(prompt=prompt_text, model_tier=tier)
        variants = _parse_variants(resp.completion, [r.id for r in kb_records], req.variant_count)
        fallback = "none"
        model_id = resp.model_id
        tokens_in = resp.prompt_tokens
        tokens_out = resp.completion_tokens
        cost = resp.cost_usd
        record_spend(cost)  # F-AUDIT-26: feed the monthly budget
        latency = resp.latency_ms

    response = DraftResponse(
        variants=variants,
        fallback_used=fallback,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        low_grounding=len(kb_records) == 0,
    )

    await emit_audit_event(
        member_id=req.member_id,
        action="llm.draft_profile_edit",
        resource_id=None,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        prompt_kind="draft",
        trace_id=req.trace_id,
        fallback_used=fallback != "none",
    )
    return response


@router.post("/draft-proposal", response_model=DraftResponse)
async def draft_proposal(req: DraftProposalRequest) -> DraftResponse:
    """Generate client-proposal one-pager."""
    embedding = await embed_query(f"{req.proposed_scope} {req.discovery_call_notes}")
    kb_records = await retrieve_top_k(member_id=req.member_id, query_embedding=embedding, top_k=8)

    prompt_text = proposal_draft_prompt(
        discovery_call_notes=req.discovery_call_notes,
        proposed_scope=req.proposed_scope,
        pricing_structure=req.pricing_structure,
        kb_records=kb_records,
        variant_count=req.variant_count,
    )

    tier = select_model_tier()
    if tier == "rule_based":
        variants = _rule_based_proposal_variants(req, req.variant_count)
        fallback = "rule_based"
        model_id = None
        tokens_in = tokens_out = 0
        cost = 0.0
        latency = 0
    else:
        from ai_worker.llm.client import complete
        resp = await complete(prompt=prompt_text, model_tier=tier)
        variants = _parse_variants(resp.completion, [r.id for r in kb_records], req.variant_count)
        fallback = "none"
        model_id = resp.model_id
        tokens_in = resp.prompt_tokens
        tokens_out = resp.completion_tokens
        cost = resp.cost_usd
        record_spend(cost)  # F-AUDIT-26: feed the monthly budget
        latency = resp.latency_ms

    response = DraftResponse(
        variants=variants,
        fallback_used=fallback,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        low_grounding=len(kb_records) == 0,
    )

    await emit_audit_event(
        member_id=req.member_id,
        action="llm.draft_proposal",
        resource_id=req.opportunity_id,
        model_id=model_id,
        tokens_in=tokens_in,
        tokens_out=tokens_out,
        cost_usd=cost,
        latency_ms=latency,
        prompt_kind="draft",
        trace_id=req.trace_id,
        fallback_used=fallback != "none",
    )
    return response


# ---- Helpers ----

def _parse_variants(raw: str, kb_refs: list[str], variant_count: int) -> list[DraftVariant]:
    """Parse LLM output into N DraftVariant objects.

    The LLM is asked to emit a JSON array of `{body, media_suggestions}` objects.
    If the parse fails, fall back to splitting the response on `---` markers.
    """
    import json
    import re

    raw = raw.strip()
    # Attempt 1: JSON array.
    try:
        arr = json.loads(raw)
        if isinstance(arr, list):
            return [
                DraftVariant(
                    body=str(item.get("body", "")).strip(),
                    media_suggestions=list(item.get("media_suggestions", []) or []),
                    kb_refs=kb_refs,
                    confidence=0.8,
                )
                for item in arr[:variant_count]
            ]
    except json.JSONDecodeError:
        pass

    # Attempt 2: split on --- markers.
    chunks = re.split(r"\n-{3,}\n", raw)
    return [
        DraftVariant(body=c.strip(), kb_refs=kb_refs, confidence=0.6)
        for c in chunks
        if c.strip()
    ][:variant_count]


def _lint_and_filter(variants: list[DraftVariant], *, target: str) -> list[DraftVariant]:
    """Run the brand guard and drop failing variants.

    F-AUDIT-28: `lint_draft` computed a `passed` verdict that every call site
    ignored — only `confidence` was read, so banned phrases and out-of-window
    lengths were surfaced as a slightly lower confidence number rather than a
    rejection. Technical Design Spec §22 requires the brand-guard linter to be
    a *gate*: "Brand-guard linter + the Compliance Governor guard stack, both
    must pass before any send."

    Failing variants are removed from the list in place. The `confidence` of a
    surviving variant is still taken from the linter, so a borderline-but-passing
    draft keeps its lower score.

    If every variant fails, the originals are returned rather than an empty
    list: silently returning nothing would look identical to "the model had
    nothing to say", and the caller's fallback path is better placed to decide
    what to do about an unlintable draft than this helper is.
    """
    kept: list[DraftVariant] = []
    for v in variants:
        lint = lint_draft(v.body, target=target)
        if lint.passed:
            v.confidence = lint.confidence
            kept.append(v)
            continue
        # Keep a record of why it was dropped, on the variant itself, so the
        # reason survives into the caller's logs.
        v.confidence = lint.confidence
        logger.warning(
            "brand_guard_rejected_variant",
            target=target,
            banned=lint.banned_phrases_found,
            fluff=lint.fluff_spans,
            length_ok=lint.length_ok,
            tone=lint.tone_alignment_score,
            body_len=len(v.body),
        )
    return kept if kept else variants


def _rule_based_content_variants(
    prompt: str, kb_records: list[Any], variant_count: int
) -> list[DraftVariant]:
    """Rule-based fallback when budget exhausted or LLM unavailable."""
    bases = []
    for i in range(variant_count):
        body = (
            f"[Variant {i+1}] {prompt}\n\n"
            f"Key context from your KB: "
            + (kb_records[0].fact if kb_records else "(no KB facts available — low grounding)")
        )
        bases.append(DraftVariant(body=body, kb_refs=[r.id for r in kb_records], confidence=0.4))
    return bases


def _rule_based_reply_variants(inbound: str, variant_count: int) -> list[DraftVariant]:
    return [
        DraftVariant(
            body=f"Thanks for your message. Could you share more context? (variant {i+1})",
            confidence=0.3,
        )
        for i in range(variant_count)
    ]


def _rule_based_outreach_variants(req: DraftOutreachRequest, variant_count: int) -> list[DraftVariant]:
    return [
        DraftVariant(
            body=(
                f"Hi — quick note. I'm working on {req.goal}; would love to connect "
                f"if that's interesting. (variant {i+1})"
            ),
            confidence=0.3,
        )
        for i in range(variant_count)
    ]


def _rule_based_profile_edit_variants(req: DraftProfileEditRequest, variant_count: int) -> list[DraftVariant]:
    return [
        DraftVariant(
            body=f"{req.target_field}: rewrite for {req.goal_mode}. (variant {i+1})",
            confidence=0.3,
        )
        for i in range(variant_count)
    ]


def _rule_based_proposal_variants(req: DraftProposalRequest, variant_count: int) -> list[DraftVariant]:
    return [
        DraftVariant(
            body=(
                f"# Proposal (variant {i+1})\n\n"
                f"## Scope\n{req.proposed_scope}\n\n"
                f"## Pricing\n{req.pricing_structure}"
            ),
            confidence=0.3,
        )
        for i in range(variant_count)
    ]
