"""Versioned prompt templates for each draft type.

Per Backend Design Concept §41: prompts are versioned; a prompt change is
a config deploy, not a code edit.
"""

from __future__ import annotations

from typing import Any


def _format_kb(records: list[Any]) -> str:
    if not records:
        return "(no KB records available — emit a low-grounding flag; do not invent facts)"
    return "\n".join(f"- [{r.category}] {r.fact}" for r in records)


def _format_voice(samples: list[Any]) -> str:
    if not samples:
        return "(no voice samples available — emit a generic tone; not personalized)"
    return "\n---\n".join(s.text for s in samples)


SYSTEM_PROMPT = """You are the OKESON-LCC AI assistant. You draft LinkedIn content for a
specific professional based on their Professional Knowledge Base.

Hard rules:
1. NEVER invent facts about the user. Cite only what appears in the KB.
2. NEVER use AI-fluff phrases like "in today's fast-paced world", "leverage",
   "synergy", "delve into", "navigate the complexities of".
3. NEVER mention you are an AI in the draft.
4. ALWAYS ground the draft in the user's KB facts below.
5. Output a JSON array of objects: [{"body": "...", "media_suggestions": []}, ...]
6. The tone should match the user's voice samples below.
"""


def content_draft_prompt(
    *,
    user_prompt: str,
    kb_records: list[Any],
    voice_samples: list[Any],
    pillar: str | None,
    format_hint: str,
    variant_count: int,
) -> str:
    return f"""{SYSTEM_PROMPT}

## Pillar
{pillar or "general"}

## Format
{format_hint}

## Variant count
{variant_count}

## User prompt
{user_prompt}

## Professional Knowledge Base (grounding source — required)
{_format_kb(kb_records)}

## Voice samples (tone source — match these)
{_format_voice(voice_samples)}

Emit exactly {variant_count} variants as a JSON array.
"""


def reply_draft_prompt(
    *,
    inbound_body: str,
    context_kind: str,
    kb_records: list[Any],
    voice_samples: list[Any],
    variant_count: int,
) -> str:
    return f"""{SYSTEM_PROMPT}

## Inbound message ({context_kind})
{inbound_body}

## Professional Knowledge Base
{_format_kb(kb_records)}

## Voice samples
{_format_voice(voice_samples)}

Emit exactly {variant_count} reply variants as a JSON array.
"""


def outreach_draft_prompt(
    *,
    persona: str,
    tone: str,
    goal: str,
    context: str,
    kb_records: list[Any],
    voice_samples: list[Any],
    variant_count: int,
) -> str:
    return f"""{SYSTEM_PROMPT}

## Persona
{persona}

## Tone
{tone}

## Goal
{goal}

## Context
{context or "(no extra context)"}

## Professional Knowledge Base
{_format_kb(kb_records)}

## Voice samples
{_format_voice(voice_samples)}

Emit exactly {variant_count} outreach variants as a JSON array. Keep the body under 400 characters
unless the goal requires more.
"""


def profile_edit_draft_prompt(
    *,
    target_field: str,
    current_value: str,
    goal_mode: str,
    user_prompt: str,
    kb_records: list[Any],
    variant_count: int,
) -> str:
    return f"""{SYSTEM_PROMPT}

## Target field
{target_field}

## Current value
{current_value}

## Goal mode
{goal_mode}

## User prompt
{user_prompt}

## Professional Knowledge Base
{_format_kb(kb_records)}

Emit exactly {variant_count} profile-edit variants as a JSON array.
"""


def proposal_draft_prompt(
    *,
    discovery_call_notes: str,
    proposed_scope: str,
    pricing_structure: str,
    kb_records: list[Any],
    variant_count: int,
) -> str:
    return f"""{SYSTEM_PROMPT}

## Discovery call notes
{discovery_call_notes}

## Proposed scope
{proposed_scope}

## Pricing structure
{pricing_structure}

## Professional Knowledge Base (case studies, achievements)
{_format_kb(kb_records)}

Emit exactly {variant_count} proposal variants as a JSON array. Each proposal should be a
one-pager Markdown structure with sections for Scope, Approach, Timeline, Pricing, and Why-me.
"""
