"""ai-worker — LLM orchestration + RAG + brand guard.

Per Backend Design Concept §41, every draft call follows:
1. Resolve KB context via RAG (top-K facts).
2. Invoke the LLM provider with cost-controlled tier.
3. Apply brand-guard linter.
4. Attach `kb_refs` to the draft (axiom 5 — grounding citation).
5. Emit `audit.event` with prompt_hash, completion_hash, model_id, tokens, cost.

Empty KB refs → emit `low_grounding=True` and let the Compliance Governor's
guard 5 deny (axiom 5)."""

__version__ = "0.1.0"
