"""Tests for compliance-types — action + risk tier + config validation."""

import pytest

from compliance_types.action import ActionType, RiskTier
from compliance_types.config import ComplianceConfig


@pytest.mark.unit
class TestActionType:
    def test_high_risk_actions(self):
        assert ActionType.CONNECTION_REQUEST.risk_tier() == RiskTier.HIGH
        assert ActionType.DM.risk_tier() == RiskTier.HIGH
        assert ActionType.JOB_APPLICATION_SUBMIT.risk_tier() == RiskTier.HIGH
        assert ActionType.EXECUTIVE_OUTREACH.risk_tier() == RiskTier.HIGH
        assert ActionType.CLIENT_PROPOSAL_SEND.risk_tier() == RiskTier.HIGH

    def test_medium_risk_actions(self):
        assert ActionType.POST_PUBLISH.risk_tier() == RiskTier.MEDIUM
        assert ActionType.SEQUENCE_STEP_SEND.risk_tier() == RiskTier.MEDIUM

    def test_low_risk_actions(self):
        assert ActionType.COMMENT_POST.risk_tier() == RiskTier.LOW
        assert ActionType.LIKE_POST.risk_tier() == RiskTier.LOW
        assert ActionType.PROFILE_VIEW.risk_tier() == RiskTier.LOW
        assert ActionType.FOLLOW_COMPANY.risk_tier() == RiskTier.LOW

    def test_all_actions_have_risk_tier(self):
        for action in ActionType:
            tier = action.risk_tier()
            assert tier in (RiskTier.LOW, RiskTier.MEDIUM, RiskTier.HIGH)


@pytest.mark.unit
class TestComplianceConfig:
    def test_h_c_weights_must_sum_to_one(self):
        cfg = ComplianceConfig(
            h_c={"weights": {"a": 0.5, "b": 0.4, "c": 0.05, "d": 0.05},
                 "warmup_threshold": 0.85, "standard_threshold": 0.65, "reserve_fraction": 0.1},
            ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
            rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
            phi={"qualified_threshold": 0.65},
        )
        # Should not raise.
        cfg.validate()

    def test_h_c_weights_sum_constraint(self):
        with pytest.raises(ValueError):
            cfg = ComplianceConfig(
                h_c={"weights": {"a": 0.5, "b": 0.3, "c": 0.1, "d": 0.05},
                     "warmup_threshold": 0.85, "standard_threshold": 0.65, "reserve_fraction": 0.1},
                ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            )
            cfg.validate()

    def test_warmup_must_exceed_standard(self):
        with pytest.raises(ValueError):
            cfg = ComplianceConfig(
                h_c={"weights": {"a": 0.7, "b": 0.3},
                     "warmup_threshold": 0.65, "standard_threshold": 0.85,
                     "reserve_fraction": 0.1},
                ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            )
            cfg.validate()

    def test_multiplier_floor_range(self):
        with pytest.raises(ValueError):
            cfg = ComplianceConfig(
                h_c={"weights": {"a": 1.0}, "warmup_threshold": 0.85,
                     "standard_threshold": 0.65, "reserve_fraction": 0.1},
                ab_d={"multiplier_floor": -0.1, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            )
            cfg.validate()

    def test_reserve_fraction_range(self):
        with pytest.raises(ValueError):
            cfg = ComplianceConfig(
                h_c={"weights": {"a": 1.0}, "warmup_threshold": 0.85,
                     "standard_threshold": 0.65, "reserve_fraction": 0.5},
                ab_d={"multiplier_floor": 0.5, "multiplier_ceiling": 2.0},
                rho={"labeled_send_threshold": 200, "vip_boost": 1.5},
                phi={"qualified_threshold": 0.65},
            )
            cfg.validate()
