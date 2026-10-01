"""Opportunity Fit Score (φ) — Python mirror of crates/compliance::phi.

```math
φ = 0.4 · S_skill + 0.25 · S_seniority + 0.20 · S_geo + 0.15 · S_comp
```

Client-track scoring substitutes `S_trigger_recency` for `S_comp`.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

DEFAULT_PHI_WEIGHTS: tuple[float, float, float, float] = (0.40, 0.25, 0.20, 0.15)


class GoalMode(str, Enum):
    JOB_HUNTING = "job_hunting"
    CLIENT_ACQUISITION = "client_acquisition"
    HYBRID = "hybrid"


@dataclass(frozen=True)
class PhiInputs:
    skill: float
    seniority: float
    geo: float
    comp: float
    trigger_recency: float
    goal_mode: GoalMode


@dataclass(frozen=True)
class PhiComponents:
    skill: float
    seniority: float
    geo: float
    comp: float
    trigger_recency: float
    goal_mode: GoalMode


def _clamp01(x: float) -> float:
    if x != x:  # NaN check
        return 0.0
    return max(0.0, min(1.0, x))


def compute_phi(
    inputs: PhiInputs,
    weights: tuple[float, float, float, float] = DEFAULT_PHI_WEIGHTS,
) -> float:
    s_skill = _clamp01(inputs.skill)
    s_seniority = _clamp01(inputs.seniority)
    s_geo = _clamp01(inputs.geo)
    s_comp = _clamp01(inputs.comp)
    s_trigger = _clamp01(inputs.trigger_recency)
    w1, w2, w3, w4 = weights

    if inputs.goal_mode == GoalMode.CLIENT_ACQUISITION:
        score = w1 * s_skill + w2 * s_seniority + w3 * s_geo + w4 * s_trigger
    else:
        score = w1 * s_skill + w2 * s_seniority + w3 * s_geo + w4 * s_comp
    return _clamp01(score)


def compute_phi_with_components(
    inputs: PhiInputs,
    weights: tuple[float, float, float, float] = DEFAULT_PHI_WEIGHTS,
) -> tuple[float, PhiComponents]:
    score = compute_phi(inputs, weights)
    components = PhiComponents(
        skill=_clamp01(inputs.skill),
        seniority=_clamp01(inputs.seniority),
        geo=_clamp01(inputs.geo),
        comp=_clamp01(inputs.comp),
        trigger_recency=_clamp01(inputs.trigger_recency),
        goal_mode=inputs.goal_mode,
    )
    return score, components
