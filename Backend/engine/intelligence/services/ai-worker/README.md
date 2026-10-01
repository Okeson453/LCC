# ai-worker

LLM orchestration + RAG + brand guard.

## Endpoints

- `POST /internal/llm/draft-content` — post / carousel / video / poll drafts.
- `POST /internal/llm/draft-reply` — engagement reply drafts.
- `POST /internal/llm/draft-outreach` — connection request / first-touch / sequence step.
- `POST /internal/llm/draft-profile-edit` — profile edit proposals.
- `POST /internal/llm/draft-proposal` — client proposal one-pagers.
- `POST /internal/llm/embed` — single-text embedding.
- `GET /healthz`, `/readyz`.

## Pipeline

```
embed(prompt) → retrieve_top_k(KB) → retrieve_voice_samples
  → build_prompt(system + KB + voice + user)
  → select_model_tier() [premium|standard|cheap|rule_based]
  → invoke LLM (or fall back to rule-based)
  → lint with brand_guard
  → emit audit.event
  → return DraftResponse{ variants, kb_refs, low_grounding, ... }
```

## Non-negotiables enforced

- §5: every draft returns `kb_refs` (≥1 required). Empty KB → `low_grounding=true`;
  the Compliance Governor's guard 5 denies (axiom 5).
- §6: rate-limit / provider-down → fall back to rule-based (NO service outage).
- §41: every LLM call emits `audit.event` with `prompt_hash`, `completion_hash`,
  `model_id`, `tokens_in`, `tokens_out`, `cost_usd`.
- §53: cost controller selects tier based on monthly usage %.

## Local dev

```bash
just bootstrap
cd engine/intelligence/services/ai-worker
uv sync
uv run python -m ai_worker
```
