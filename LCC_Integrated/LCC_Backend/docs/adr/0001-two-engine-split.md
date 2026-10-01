# ADR-0001: Two-Engine Split (Rust Core + Python Intelligence)

## Status

Accepted.

## Context

The product needs:
- A transactional, safety-critical backbone that owns DB writes, RLS, audit, idempotency.
- An LLM/ML backend that produces drafts, scores, embeddings.

Mixing LLM code into the same deployable as the transactional core would:
- Increase blast radius: a slow LLM call would stall critical-path evaluations.
- Make language fragmentation worse: most safety primitives are best written
  in Rust; most ML/DS tooling is Python.

## Decision

Split into two engines:
- **Rust Core Engine** (`engine/core/`): transactional, audit-in-tx, compliance-governor,
  integration-gateway, all domain services.
- **Python Intelligence Engine** (`engine/intelligence/`): ai-worker,
  opportunity-intel, kb-intel, voice-intel, scoring-intel, and async workers.

Cross-engine communication is **only** via:
- gRPC contracts (`proto/lcc/v1/*.proto`)
- The event bus (Redis Streams in Phase 1-2; Kafka in Phase 3+)

A CI-gated `infra/ci/scripts/check_boundaries.sh` enforces this.

## Consequences

- Two separate deployment units, two CI pipelines.
- Each engine has its own language-specific linters (clippy + ruff + mypy).
- The boundary scripts catch accidental imports early.
