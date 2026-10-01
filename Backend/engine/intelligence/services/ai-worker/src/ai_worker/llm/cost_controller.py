"""Cost-Aware Decision Engine for the AI worker (Backend Design Concept §53).

Selects the model tier based on monthly usage %:
- < 80%  → PREMIUM
- < 90%  → STANDARD
- < 100% → CHEAP
- ≥ 100% → RULE_BASED (soft stop; drafting still works via templates)
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class CostState:
    monthly_budget_usd: float
    spent_usd: float = 0.0

    def usage_pct(self) -> float:
        if self.monthly_budget_usd <= 0:
            return 0.0
        return self.spent_usd / self.monthly_budget_usd

    def record_spend(self, cost: float) -> None:
        self.spent_usd += cost


# Global per-process cost state. Per-member state would be more correct but
# the initial implementation tracks process-wide usage and emits alerts.
_state = CostState(monthly_budget_usd=500.0)


def select_model_tier() -> str:
    """Return the model tier based on current monthly usage.

    F-AUDIT-26: this method was correct, but `record_spend` was never called
    from any production code path — a repo-wide grep found only the definition
    and test files. `spent_usd` therefore stayed at 0.0 forever, `usage_pct()`
    was always 0.0, and `select_model_tier()` always returned "premium". The
    whole §53 ladder (standard at 80%, cheap at 90%, rule_based at 100%) and
    the monthly budget ceiling were unreachable, and the `cost_usd` captured
    from every LLM response was discarded rather than fed back here.

    `record_spend` is now called from the draft endpoints (see api/drafts.py).
    """
    pct = _state.usage_pct()
    if pct >= 1.0:
        return "rule_based"
    if pct >= 0.9:
        return "cheap"
    if pct >= 0.8:
        return "standard"
    return "premium"


def get_state() -> CostState:
    return _state


def configure(monthly_budget_usd: float) -> None:
    """Reset the cost state with a new monthly budget. Used at startup.

    F-AUDIT-26: this was never called at startup, so
    `Settings.monthly_budget_usd` (ai_worker/config.py) was ignored and the
    budget was hardcoded to 500.0. It is now wired in __main__.py.
    """
    global _state
    if monthly_budget_usd <= 0:
        raise ValueError(f"monthly_budget_usd must be positive, got {monthly_budget_usd}")
    _state = CostState(monthly_budget_usd=monthly_budget_usd)
