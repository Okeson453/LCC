"""Cost controller — selects model tier based on monthly usage.

Per Backend Design Concept §53:
- < 80%  → PREMIUM (best model)
- < 90%  → STANDARD (good model)
- < 100% → CHEAP (small model)
- ≥ 100% → RULE_BASED (soft stop, no service outage)
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ModelTier(str, Enum):
    PREMIUM = "premium"
    STANDARD = "standard"
    CHEAP = "cheap"
    RULE_BASED = "rule_based"


@dataclass
class CostController:
    """Tracks monthly spend and selects a model tier per call."""

    monthly_budget_usd: float
    spent_usd: float = 0.0

    def record_spend(self, cost: float) -> None:
        """Record a completed LLM call's cost."""
        self.spent_usd += max(0.0, cost)

    def usage_pct(self) -> float:
        if self.monthly_budget_usd <= 0:
            return 0.0
        return self.spent_usd / self.monthly_budget_usd

    def select_tier(self) -> ModelTier:
        pct = self.usage_pct()
        if pct >= 1.0:
            return ModelTier.RULE_BASED
        if pct >= 0.9:
            return ModelTier.CHEAP
        if pct >= 0.8:
            return ModelTier.STANDARD
        return ModelTier.PREMIUM

    def is_hard_stop(self) -> bool:
        """Return True if usage ≥ 100%; caller should fail-closed."""
        return self.usage_pct() >= 1.0
