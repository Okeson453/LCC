"""h_c — composite account-health score (Backend Design Concept §51).

h_c ∈ [0, 1] blends 5 weighted components:
- response_rate (inbound replies / outbound sends)
- acceptance_rate (connection accepts / connection sends)
- error_rate_24h (inverted)
- restriction_penalty (inverted)
- engagement_quality (avg engagement per post)

Bands: WARMUP (< warmup_threshold), STANDARD (< standard_threshold), COLD.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class HealthSignals:
    response_rate: float          # 0..1
    acceptance_rate: float        # 0..1
    error_rate_24h: float         # 0..1
    restriction_penalty: float    # 0..1
    engagement_quality: float     # 0..1

    weights: dict[str, float] | None = None
    warmup_threshold: float = 0.85
    standard_threshold: float = 0.65

    def __post_init__(self) -> None:
        if self.weights is None:
            self.weights = {
                "response_rate": 0.30,
                "acceptance_rate": 0.20,
                "error_rate_inverted": 0.20,
                "restriction_inverted": 0.15,
                "engagement_quality": 0.15,
            }


def compute_health(signals: HealthSignals) -> tuple[float, str]:
    """Return (h_c, band)."""
    w = signals.weights or {}
    score = (
        w.get("response_rate", 0.3) * signals.response_rate
        + w.get("acceptance_rate", 0.2) * signals.acceptance_rate
        + w.get("error_rate_inverted", 0.2) * (1.0 - signals.error_rate_24h)
        + w.get("restriction_inverted", 0.15) * (1.0 - signals.restriction_penalty)
        + w.get("engagement_quality", 0.15) * signals.engagement_quality
    )
    score = max(0.0, min(1.0, score))

    if score >= signals.warmup_threshold:
        band = "warmup"
    elif score >= signals.standard_threshold:
        band = "standard"
    else:
        band = "cold"
    return score, band


__all__ = ["HealthSignals", "compute_health"]
