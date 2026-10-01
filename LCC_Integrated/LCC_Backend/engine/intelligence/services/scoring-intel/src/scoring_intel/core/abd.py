"""ab_d — adaptive daily-pacing multiplier (Backend Design Concept §52).

ab_d is a stateful scaling factor applied to daily_cap, currently ∈ [0.5, 2.0].
It adapts based on observed signals (acceptance rate, error rate, restriction events).
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class AbdState:
    multiplier: float = 1.0
    observed_acceptance_rate: float = 0.0
    observed_error_rate: float = 0.0
    restriction_count_24h: int = 0
    floor: float = 0.5
    ceiling: float = 2.0

    history: list[float] = field(default_factory=list)


def update(
    state: AbdState,
    *,
    acceptance_rate: float | None = None,
    error_rate: float | None = None,
    restriction_count_24h: int | None = None,
    base_daily_cap: int,
) -> AbdState:
    """Update the multiplier based on observed signals."""
    if acceptance_rate is not None:
        state.observed_acceptance_rate = acceptance_rate
    if error_rate is not None:
        state.observed_error_rate = error_rate
    if restriction_count_24h is not None:
        state.restriction_count_24h = restriction_count_24h

    multiplier = 1.0
    if state.observed_acceptance_rate < 0.2:
        multiplier *= 0.7
    if state.observed_error_rate > 0.10:
        multiplier *= 0.8
    if state.restriction_count_24h > 0:
        multiplier *= 0.6

    # Clamp.
    multiplier = max(state.floor, min(state.ceiling, multiplier))
    state.multiplier = multiplier
    state.history.append(multiplier)
    if len(state.history) > 200:
        state.history = state.history[-200:]

    return state


def effective_daily_cap(state: AbdState, base_daily_cap: int) -> int:
    """Apply the multiplier to the base daily cap."""
    return int(round(base_daily_cap * state.multiplier))


__all__ = ["AbdState", "update", "effective_daily_cap"]
