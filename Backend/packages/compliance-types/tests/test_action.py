"""Tests for compliance-types — action + risk tier + config validation.

The 5-tier taxonomy and the flat `ComplianceConfig` shape mirror
`crates/compliance::{action,config}`; these tests pin that mirror.
"""

import pytest

from compliance_types.action import ActionType, RiskTier, action_risk_tier
from compliance_types.config import ComplianceConfig


def _valid_config(**overrides: object) -> ComplianceConfig:
    """A minimal valid config; override individual fields per test."""
    fields: dict[str, object] = {
        "h_c_weights": (0.35, 0.25, 0.20, 0.20),
        "phi_weights": (0.40, 0.25, 0.20, 0.15),
        "h_c_warmup_threshold": 0.4,
        "h_c_standard_threshold": 0.7,
        "reserve_fraction": 0.15,
        "ab_d_multiplier_floor": 0.3,
        "phi_qualification_threshold": 0.55,
    }
    fields.update(overrides)
    return ComplianceConfig(**fields)  # type: ignore[arg-type]


@pytest.mark.unit
class TestActionType:
    def test_high_stakes_actions(self):
        assert ActionType.CONNECTION_REQUEST.risk_tier() == RiskTier.TIER_3_NETWORK_GROWTH
        assert ActionType.DIRECT_MESSAGE.risk_tier() == RiskTier.TIER_4_ONE_TO_ONE_OUTREACH
        assert ActionType.SEQUENCE_STEP_SEND.risk_tier() == RiskTier.TIER_4_ONE_TO_ONE_OUTREACH
        assert ActionType.JOB_APPLICATION_SUBMIT.risk_tier() == RiskTier.TIER_5_HIGH_STAKES
        assert ActionType.EXECUTIVE_OUTREACH.risk_tier() == RiskTier.TIER_5_HIGH_STAKES
        assert ActionType.CLIENT_PROPOSAL_SEND.risk_tier() == RiskTier.TIER_5_HIGH_STAKES

    def test_medium_risk_actions(self):
        assert ActionType.COMMENT.risk_tier() == RiskTier.TIER_2_LIGHT_ENGAGEMENT
        assert ActionType.LIKE.risk_tier() == RiskTier.TIER_2_LIGHT_ENGAGEMENT
        assert ActionType.POST_PUBLISH.risk_tier() == RiskTier.TIER_2_LIGHT_ENGAGEMENT
        assert ActionType.PROFILE_EDIT_SUBMIT.risk_tier() == RiskTier.TIER_2_LIGHT_ENGAGEMENT

    def test_low_risk_actions(self):
        assert ActionType.CONTENT_DRAFT.risk_tier() == RiskTier.TIER_1_DRAFT_OR_EDIT
        assert ActionType.PROFILE_EDIT_DRAFT.risk_tier() == RiskTier.TIER_1_DRAFT_OR_EDIT
        assert ActionType.KB_RECORD_CREATE.risk_tier() == RiskTier.TIER_1_DRAFT_OR_EDIT
        assert ActionType.RESEARCH_SCAN.risk_tier() == RiskTier.TIER_1_DRAFT_OR_EDIT

    def test_all_actions_have_risk_tier(self):
        for action in ActionType:
            tier = action.risk_tier()
            assert isinstance(tier, RiskTier)
            assert tier == action_risk_tier(action)

    def test_risk_tier_minimum_h_c(self):
        assert RiskTier.TIER_1_DRAFT_OR_EDIT.minimum_h_c == 0.0
        assert RiskTier.TIER_2_LIGHT_ENGAGEMENT.minimum_h_c == 0.3
        assert RiskTier.TIER_3_NETWORK_GROWTH.minimum_h_c == 0.4
        assert RiskTier.TIER_4_ONE_TO_ONE_OUTREACH.minimum_h_c == 0.5
        assert RiskTier.TIER_5_HIGH_STAKES.minimum_h_c == 0.5


@pytest.mark.unit
class TestComplianceConfig:
    def test_h_c_weights_must_sum_to_one(self):
        # Should not raise.
        _valid_config().validate()

    def test_h_c_weights_sum_constraint(self):
        with pytest.raises(ValueError):
            _valid_config(h_c_weights=(0.5, 0.3, 0.1, 0.05))

    def test_phi_weights_sum_constraint(self):
        with pytest.raises(ValueError):
            _valid_config(phi_weights=(0.5, 0.3, 0.1, 0.05))

    def test_warmup_must_exceed_standard(self):
        with pytest.raises(ValueError):
            _valid_config(h_c_warmup_threshold=0.85, h_c_standard_threshold=0.65).validate()

    def test_multiplier_floor_range(self):
        with pytest.raises(ValueError):
            _valid_config(ab_d_multiplier_floor=-0.1).validate()

    def test_reserve_fraction_range(self):
        with pytest.raises(ValueError):
            _valid_config(reserve_fraction=0.5).validate()

    def test_phi_qualification_threshold_range(self):
        with pytest.raises(ValueError):
            _valid_config(phi_qualification_threshold=1.5).validate()

    def test_defaults_are_valid(self):
        ComplianceConfig().validate()
