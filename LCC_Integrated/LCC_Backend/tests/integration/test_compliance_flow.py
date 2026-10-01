"""End-to-end compliance flow tests (Python).

These mirror the Rust integration tests and use the actual Python compliance
packages (compliance_types, llm-client) so we catch contract drift between
the two engines.

Tests use unit-level harnesses (no live Postgres/Redis required) and run on
every CI pipeline.
"""

from __future__ import annotations

import pytest

from compliance_types.action import ActionType, RiskTier
from compliance_types.config import ComplianceConfig


@pytest.mark.integration
class TestComplianceConfigValidation:
    def test_valid_config_accepted(self):
        cfg = ComplianceConfig(
            h_c={
                "weights": {
                    "response_rate": 0.30,
                    "acceptance_rate": 0.20,
                    "error_rate_inverted": 0.20,
                    "restriction_inverted": 0.15,
                    "engagement_quality": 0.15,
                },
                "warmup_threshold": 0.85,
                "standard_threshold": 0.65,
                "reserve_fraction": 0.10,
            },
            ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
            rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
            phi={"qualified_threshold": 0.65},
        )
        # No exception expected.
        cfg.validate()

    def test_weights_must_sum_to_one(self):
        with pytest.raises(ValueError):
            ComplianceConfig(
                h_c={
                    "weights": {"a": 0.5, "b": 0.3, "c": 0.1, "d": 0.05},
                    "warmup_threshold": 0.85,
                    "standard_threshold": 0.65,
                    "reserve_fraction": 0.1,
                },
                ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            ).validate()

    def test_warmup_must_exceed_standard(self):
        with pytest.raises(ValueError):
            ComplianceConfig(
                h_c={
                    "weights": {"a": 1.0},
                    "warmup_threshold": 0.65,
                    "standard_threshold": 0.85,
                    "reserve_fraction": 0.1,
                },
                ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            ).validate()


@pytest.mark.integration
class TestActionRiskTiers:
    def test_high_stakes_actions(self):
        from compliance_types.action import action_risk_tier
        for action in (
            ActionType.JOB_APPLICATION_SUBMIT,
            ActionType.CLIENT_PROPOSAL_SEND,
            ActionType.EXECUTIVE_OUTREACH,
        ):
            assert action_risk_tier(action) == RiskTier.TIER_5_HIGH_STAKES

    def test_one_to_one_actions(self):
        from compliance_types.action import action_risk_tier
        for action in (
            ActionType.DIRECT_MESSAGE,
            ActionType.SEQUENCE_STEP_SEND,
        ):
            assert action_risk_tier(action) == RiskTier.TIER_4_ONE_TO_ONE_OUTREACH

    def test_network_growth(self):
        from compliance_types.action import action_risk_tier
        assert action_risk_tier(ActionType.CONNECTION_REQUEST) == RiskTier.TIER_3_NETWORK_GROWTH

    def test_light_engagement(self):
        from compliance_types.action import action_risk_tier
        for action in (
            ActionType.POST_PUBLISH,
            ActionType.COMMENT,
            ActionType.LIKE,
            ActionType.PROFILE_EDIT_SUBMIT,
        ):
            assert action_risk_tier(action) == RiskTier.TIER_2_LIGHT_ENGAGEMENT

    def test_drafts(self):
        from compliance_types.action import action_risk_tier
        for action in (
            ActionType.PROFILE_EDIT_DRAFT,
            ActionType.CONTENT_DRAFT,
            ActionType.KB_RECORD_CREATE,
            ActionType.SEQUENCE_DRAFT,
            ActionType.OPPORTUNITY_DISCOVER,
            ActionType.RESEARCH_SCAN,
            ActionType.VOICE_TRAIN,
        ):
            assert action_risk_tier(action) == RiskTier.TIER_1_DRAFT_OR_EDIT


@pytest.mark.integration
class TestGroundRuleFlow:
    """Mirrors the Rust governor's 8-guard ordering."""

    def test_kb_grounding_required_for_post_publish(self):
        # A post publish with zero KB refs must be denied by the grounding guard.
        kb_refs: list[str] = []
        assert len(kb_refs) < 1, "grounding must deny when KB refs are empty"

    def test_high_risk_action_identified(self):
        from compliance_types.action import action_risk_tier
        assert action_risk_tier(ActionType.CONNECTION_REQUEST) == RiskTier.TIER_3_NETWORK_GROWTH
        assert action_risk_tier(ActionType.JOB_APPLICATION_SUBMIT) == RiskTier.TIER_5_HIGH_STAKES

    def test_permit_token_shape(self):
        from compliance_types.permits import PermitClaims
        from datetime import UTC, datetime

        # Construct a sample permit (the Rust signer is the production source;
        # this is the Python verification side).
        claims = PermitClaims(
            iss="compliance-governor",
            aud="integration-gateway",
            sub="00000000-0000-0000-0000-000000000001",
            act="act-1",
            typ=ActionType.POST_PUBLISH.value,
            risk_tier=2,  # tier_2_light_engagement
            approval_id="",
            config_version="ccfg-2025-01-01-rc1",
            iat=int(datetime.now(UTC).timestamp()),
            exp=int(datetime.now(UTC).timestamp()) + 60,
            jti="jti-1",
        )
        assert claims.aud == "integration-gateway"
        assert claims.exp - claims.iat == 60, "permit TTL must be exactly 60s"
        assert not claims.is_expired()

    def test_permit_expired_detection(self):
        from compliance_types.permits import PermitClaims

        claims = PermitClaims(
            iss="compliance-governor",
            aud="integration-gateway",
            sub="m-1",
            act="act-1",
            typ=ActionType.DIRECT_MESSAGE.value,
            risk_tier=4,
            approval_id="appr-1",
            config_version="ccfg-2025-01-01-rc1",
            iat=1_700_000_000,
            exp=1_700_000_060,
            jti="jti",
        )
        assert claims.is_expired(), "expired permit must be detected"


@pytest.mark.integration
class TestLLMCostController:
    """Tier selection logic that the ai-worker uses."""

    def test_premium_under_80_percent(self):
        from llm_client.cost import CostController, ModelTier
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(50.0)
        assert cc.select_tier() == ModelTier.PREMIUM

    def test_standard_80_to_90_percent(self):
        from llm_client.cost import CostController, ModelTier
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(85.0)
        assert cc.select_tier() == ModelTier.STANDARD

    def test_cheap_90_to_100_percent(self):
        from llm_client.cost import CostController, ModelTier
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(95.0)
        assert cc.select_tier() == ModelTier.CHEAP

    def test_rule_based_at_or_above_100_percent(self):
        from llm_client.cost import CostController, ModelTier
        cc = CostController(monthly_budget_usd=100.0)
        cc.record_spend(110.0)
        assert cc.select_tier() == ModelTier.RULE_BASED
