"""LLM client tests."""

import pytest

from llm_client.base import LLMClient, LLMMessage, LLMRequest, LLMResponse
from llm_client.cost import CostController, ModelTier


@pytest.mark.unit
class TestLLMRequest:
    def test_request_creation(self):
        req = LLMRequest(
            model_id="gpt-4o",
            messages=[LLMMessage(role="user", content="hello")],
            temperature=0.5,
            max_tokens=100,
        )
        assert req.temperature == 0.5
        assert len(req.messages) == 1


@pytest.mark.unit
class TestCostController:
    def test_initial_state(self):
        cc = CostController(monthly_budget_usd=1000.0)
        assert cc.monthly_budget_usd == 1000.0
        assert cc.spent_usd == 0.0

    def test_record_spend(self):
        cc = CostController(monthly_budget_usd=1000.0)
        cc.record_spend(50.0)
        cc.record_spend(50.0)
        assert cc.spent_usd == 100.0

    def test_select_tier_premium(self):
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(50.0)
        assert cc.select_tier() == ModelTier.PREMIUM

    def test_select_tier_standard(self):
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(85.0)
        assert cc.select_tier() == ModelTier.STANDARD

    def test_select_tier_cheap(self):
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(95.0)
        assert cc.select_tier() == ModelTier.CHEAP

    def test_select_tier_rule_based(self):
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(120.0)
        assert cc.select_tier() == ModelTier.RULE_BASED

    def test_boundary_at_80_pct(self):
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(79.0)
        assert cc.select_tier() == ModelTier.PREMIUM
        cc.record_spend(1.0)  # 80%
        assert cc.select_tier() == ModelTier.STANDARD


@pytest.mark.unit
class TestLLMResponse:
    def test_response_creation(self):
        resp = LLMResponse(
            completion="hello world",
            model_id="gpt-4o",
            prompt_tokens=10,
            completion_tokens=2,
            cost_usd=0.0001,
            latency_ms=200,
        )
        assert resp.completion == "hello world"
        assert resp.model_id == "gpt-4o"
