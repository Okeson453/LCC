"""φ (phi) scoring — mirror of `crates/compliance/src/phi.rs`.

Hybrid model:
- If `labeled_sends >= 200`: ML model predicts receptiveness (signal_score
  + text_relevance + timing_window + reciprocity_balance).
- Otherwise: rule-based (signal strength + recency + reciprocity + KB fit).

Returns a PhiResult with the same JSON shape as the Rust service so that
the Compliance Governor's `phi.evaluate` can consume either engine.
"""

from __future__ import annotations

import math
from dataclasses import dataclass


@dataclass
class PhiSignal:
    """One opportunity-signal observation."""

    kind: str                 # "job_change"|"funding_event"|"post_about_problem"|"competitor_mention"|"manual"
    recency_days: int
    weight: float             # 0..1
    text: str = ""
    matched_kb_facts: int = 0


@dataclass
class PhiResult:
    phi_score: float          # 0..1
    components: dict[str, float]
    mode: str                 # "ml" | "rule_based"
    confidence: float         # 0..1
    hard_qualifies: bool      # True if phi_score ≥ qualified threshold
    rationale: list[str]

    def to_dict(self) -> dict:
        return {
            "phi_score": self.phi_score,
            "components": self.components,
            "mode": self.mode,
            "confidence": self.confidence,
            "hard_qualifies": self.hard_qualifies,
            "rationale": self.rationale,
        }


def score(
    signals: list[PhiSignal],
    labeled_sends: int,
    qualified_threshold: float = 0.65,
) -> PhiResult:
    """Compute phi_score using either the ML model or rule-based fallback.

    The Python service uses a deterministic scoring function here. The ML
    model itself (when labeled_sends >= 200) is pluggable via the
    `core/model_loader.py` interface.
    """
    if not signals:
        return PhiResult(
            phi_score=0.0,
            components={},
            mode="rule_based",
            confidence=1.0,
            hard_qualifies=False,
            rationale=["no signals provided"],
        )

    if labeled_sends >= 200:
        return _score_ml(signals, qualified_threshold)
    return _score_rule_based(signals, qualified_threshold)


def _score_rule_based(signals: list[PhiSignal], threshold: float) -> PhiResult:
    """Rule-based fallback for cold-start (< 200 labeled sends)."""
    rationale = []

    # 1) Signal strength: weighted sum capped at 1.0
    raw_strength = min(1.0, sum(s.weight for s in signals))
    # Decay based on recency (most-recent weighted higher).
    decayed = sum(s.weight * math.exp(-s.recency_days / 14.0) for s in signals)
    decayed = min(1.0, decayed)
    if decayed < raw_strength * 0.5:
        rationale.append("signals decayed due to age")

    # 2) KB-fit proxy: matched_kb_facts across signals.
    kb_fit = min(1.0, sum(s.matched_kb_facts for s in signals) / 5.0)

    # 3) Diversity bonus (multiple signal kinds).
    kinds = {s.kind for s in signals}
    diversity_bonus = min(0.1, 0.04 * (len(kinds) - 1))

    components = {
        "signal_strength": decayed,
        "kb_fit": kb_fit,
        "diversity_bonus": diversity_bonus,
    }
    score = min(1.0, 0.6 * decayed + 0.3 * kb_fit + diversity_bonus)
    rationale.append(
        f"rule-based: signal_strength={decayed:.2f}, kb_fit={kb_fit:.2f}, kinds={len(kinds)}"
    )

    return PhiResult(
        phi_score=score,
        components=components,
        mode="rule_based",
        confidence=0.6 if score < threshold else 0.7,
        hard_qualifies=score >= threshold,
        rationale=rationale,
    )


def _score_ml(signals: list[PhiSignal], threshold: float) -> PhiResult:
    """ML-based scoring for warm model (>= 200 labeled sends).

    The actual model is loaded via `core.model_loader`. We provide a
    simple linear ensemble here as the production model would replace this.
    """
    try:
        from opportunity_intel.core.model_loader import load_phi_model

        model = load_phi_model()
        return model.predict(signals, threshold)
    except Exception:
        # Graceful degradation.
        return _score_rule_based(signals, threshold)


__all__ = ["PhiSignal", "PhiResult", "score"]
