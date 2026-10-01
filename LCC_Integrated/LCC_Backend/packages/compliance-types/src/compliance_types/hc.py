"""Account Health Composite (H_c) — Python mirror of crates/compliance::h_c.

```math
H_c = w_1 · A_r + w_2 · R_r + w_3 · (1 - Q_u) + w_4 · T_a
```

NaN inputs → H_c=0 (fail-safe strictest caps).
"""

from __future__ import annotations

import math
from dataclasses import dataclass

DEFAULT_HC_WEIGHTS: tuple[float, float, float, float] = (0.35, 0.25, 0.20, 0.20)


@dataclass(frozen=True)
class HcInputs:
    """Inputs to H_c computation. All values are bounded to [0,1]."""

    acceptance_rate: float      # A_r — connection-request acceptance rate, trailing 30d
    reply_rate: float           # R_r — outreach reply rate, trailing 30d
    quota_utilization: float    # Q_u — quota-utilization ratio, trailing 7d
    tenure_factor: float        # T_a — account tenure factor, min(1, days_active/90)


@dataclass(frozen=True)
class HcComponents:
    """Per-component breakdown for observability."""

    acceptance_rate: float
    reply_rate: float
    quota_utilization: float
    tenure_factor: float
    weights: tuple[float, float, float, float]


def _clamp01(x: float) -> float:
    if math.isnan(x):
        return 0.0
    return max(0.0, min(1.0, x))


def compute_h_c(
    inputs: HcInputs,
    weights: tuple[float, float, float, float] = DEFAULT_HC_WEIGHTS,
) -> float:
    """Compute H_c per Source Technical Design Spec §4.

    Returns 0.0 if any input is NaN (fail-safe). Inputs are clamped to [0,1].
    """
    a_r = _clamp01(inputs.acceptance_rate)
    r_r = _clamp01(inputs.reply_rate)
    q_u = _clamp01(inputs.quota_utilization)
    t_a = _clamp01(inputs.tenure_factor)
    w1, w2, w3, w4 = weights
    result = w1 * a_r + w2 * r_r + w3 * (1.0 - q_u) + w4 * t_a
    return _clamp01(result)


def compute_h_c_with_components(
    inputs: HcInputs,
    weights: tuple[float, float, float, float] = DEFAULT_HC_WEIGHTS,
) -> tuple[float, HcComponents]:
    """Compute H_c and return per-component breakdown."""
    score = compute_h_c(inputs, weights)
    components = HcComponents(
        acceptance_rate=_clamp01(inputs.acceptance_rate),
        reply_rate=_clamp01(inputs.reply_rate),
        quota_utilization=_clamp01(inputs.quota_utilization),
        tenure_factor=_clamp01(inputs.tenure_factor),
        weights=weights,
    )
    return score, components
