"""Reply-Probability Estimate (ρ) — Python mirror of crates/compliance::rho.

CRITICAL: ρ coefficients (β) are NOT shipped with production defaults.
Until ≥200 labeled sends exist for a member, the system MUST fall back to
rule-based priority order (VIP tag > recency > mutual count), stated
explicitly rather than substituting an uncalibrated model silently.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

MIN_LABELED_SENDS: int = 200


class RhoMode(str, Enum):
    """Which estimator produced the ρ score."""

    MODEL_INFERENCE = "model_inference"
    RULE_BASED_FALLBACK = "rule_based_fallback"


@dataclass(frozen=True)
class ReplyProbabilityFeatures:
    member_id: str
    contact_id: str
    mutual_count: int
    personalization_score: float
    prior_interaction_flag: bool
    contact_tier: str
    last_interaction_age_days: int
    tag_match_flags: tuple[str, ...]


@dataclass(frozen=True)
class RhoResult:
    rho: float
    mode: RhoMode
    labeled_send_count: int
    model_version: str | None
    reason: str


def _logistic(x: float) -> float:
    if x >= 0.0:
        return 1.0 / (1.0 + math.exp(-x))
    ex = math.exp(x)
    return ex / (1.0 + ex)


def predict_reply_probability(
    features: ReplyProbabilityFeatures,
    betas: tuple[float, float, float, float] | None,
    labeled_send_count: int,
    model_version: str | None = None,
) -> RhoResult:
    """Predict ρ — falls back to rule-based if sample size < MIN_LABELED_SENDS.

    If `labeled_send_count >= MIN_LABELED_SENDS`, the calibrated logistic
    model is used; otherwise rule-based fallback is used and `mode` reflects
    that fact to the caller (for transparency).
    """
    if labeled_send_count < MIN_LABELED_SENDS:
        rule_score = _rule_based_score(features)
        return RhoResult(
            rho=rule_score,
            mode=RhoMode.RULE_BASED_FALLBACK,
            labeled_send_count=labeled_send_count,
            model_version=model_version,
            reason=f"labeled_send_count={labeled_send_count} < {MIN_LABELED_SENDS}; rule-based fallback in effect",
        )
    if betas is None:
        raise ValueError("calibrated model required when labeled_send_count >= MIN_LABELED_SENDS")
    prior = 1.0 if features.prior_interaction_flag else 0.0
    x = betas[0] + betas[1] * features.mutual_count + betas[2] * features.personalization_score + betas[3] * prior
    return RhoResult(
        rho=_logistic(x),
        mode=RhoMode.MODEL_INFERENCE,
        labeled_send_count=labeled_send_count,
        model_version=model_version,
        reason="calibrated model inference",
    )


def _rule_based_score(features: ReplyProbabilityFeatures) -> float:
    """VIP > recency > mutual count."""
    tier_score = {
        "VIP": 1.0,
        "standard": 0.5,
        "peer": 0.25,
    }.get(features.contact_tier, 0.1)

    age = features.last_interaction_age_days
    if age <= 7:
        recency = 1.0
    elif age <= 30:
        recency = 0.5
    elif age <= 90:
        recency = 0.25
    else:
        recency = 0.0

    mutual = min(1.0, features.mutual_count / 25.0)
    return 0.50 * tier_score + 0.30 * recency + 0.20 * mutual


import math  # for _logistic — placed at end to satisfy import order
