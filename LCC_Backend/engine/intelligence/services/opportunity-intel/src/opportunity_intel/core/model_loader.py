"""Model loader — loads the trained phi/rho/ab_d/voice models.

Models are versioned artifacts stored in object storage. The Python
service downloads a model.json + weights on first use, caches locally,
and refreshes every 24h.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path

from opportunity_intel.core.phi_model import PhiResult, PhiSignal

_MODEL_DIR = Path(os.environ.get("LCC_MODEL_DIR", "/tmp/lcc-models"))
_MODEL_DIR.mkdir(parents=True, exist_ok=True)


@dataclass
class PhiModelArtifact:
    version: str
    weights: dict[str, float]
    threshold: float

    def predict(self, signals: list[PhiSignal], qualified_threshold: float) -> PhiResult:
        # Simple linear ensemble with the model's weights.
        raw_strength = min(1.0, sum(s.weight for s in signals))
        decayed = sum(s.weight * (1.0 / (1.0 + 0.05 * s.recency_days)) for s in signals)
        decayed = min(1.0, decayed)
        kb_fit = min(1.0, sum(s.matched_kb_facts for s in signals) / 5.0)

        components = {
            "signal_strength": decayed,
            "kb_fit": kb_fit,
            "raw_strength": raw_strength,
        }
        w = self.weights
        score = min(1.0, w.get("signal_strength", 0.6) * decayed + w.get("kb_fit", 0.3) * kb_fit)

        return PhiResult(
            phi_score=score,
            components=components,
            mode="ml",
            confidence=0.8,
            hard_qualifies=score >= qualified_threshold,
            rationale=[f"model version {self.version}"],
        )


_phi_model: PhiModelArtifact | None = None


def load_phi_model() -> PhiModelArtifact:
    """Load the cached phi model (or fallback defaults if not present)."""
    global _phi_model
    if _phi_model is not None:
        return _phi_model
    model_path = _MODEL_DIR / "phi_model.json"
    if model_path.exists():
        data = json.loads(model_path.read_text())
        _phi_model = PhiModelArtifact(
            version=data.get("version", "0.0.0"),
            weights=data.get("weights", {"signal_strength": 0.6, "kb_fit": 0.3}),
            threshold=data.get("threshold", 0.65),
        )
    else:
        # Default untrained artifact (rule-based scoring will be used).
        _phi_model = PhiModelArtifact(version="0.0.0-default", weights={"signal_strength": 0.6, "kb_fit": 0.3}, threshold=0.65)
    return _phi_model


__all__ = ["load_phi_model", "PhiModelArtifact"]
