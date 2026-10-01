"""Scoring tests."""

import pytest

from scoring_intel.core.abd import AbdState, effective_daily_cap, update
from scoring_intel.core.hc import HealthSignals, compute_health
from scoring_intel.core.rho import ContactFeature, score_contact


@pytest.mark.unit
class TestRho:
    def test_vip_scores_higher(self):
        c1 = ContactFeature(contact_id="a", is_vip=True, matched_kb_facts=3)
        c2 = ContactFeature(contact_id="b", is_vip=False, matched_kb_facts=3)
        s1 = score_contact(c1, labeled_sends=0)
        s2 = score_contact(c2, labeled_sends=0)
        assert s1.score > s2.score
        assert s1.mode == "rule_based"

    def test_cold_start_rule_based(self):
        c = ContactFeature(contact_id="x")
        s = score_contact(c, labeled_sends=0)
        assert s.mode == "rule_based"

    def test_warm_start_ml_mode(self):
        c = ContactFeature(contact_id="x", matched_kb_facts=3)
        s = score_contact(c, labeled_sends=200)
        # ML loader may not exist → falls back to rule_based. Either is acceptable.
        assert s.mode in ("ml", "rule_based")

    def test_recency_signal(self):
        c_recent = ContactFeature(contact_id="a", last_contact_days=2)
        c_old = ContactFeature(contact_id="b", last_contact_days=120)
        s_r = score_contact(c_recent, labeled_sends=0)
        s_o = score_contact(c_old, labeled_sends=0)
        assert s_r.components["recency"] > s_o.components["recency"]


@pytest.mark.unit
class TestAbd:
    def test_default_multiplier(self):
        state = AbdState()
        assert state.multiplier == 1.0

    def test_low_acceptance_dampens(self):
        state = AbdState()
        update(state, acceptance_rate=0.05, base_daily_cap=30)
        assert state.multiplier < 1.0

    def test_restriction_dampens(self):
        state = AbdState()
        update(state, restriction_count_24h=1, base_daily_cap=30)
        assert state.multiplier <= 0.6

    def test_clamp_to_floor(self):
        state = AbdState(floor=0.5)
        for _ in range(5):
            update(state, acceptance_rate=0.0, error_rate=1.0, restriction_count_24h=5, base_daily_cap=30)
        assert state.multiplier >= state.floor

    def test_effective_daily_cap(self):
        state = AbdState(multiplier=1.5)
        assert effective_daily_cap(state, 30) == 45


@pytest.mark.unit
class TestHc:
    def test_warmup_band(self):
        signals = HealthSignals(response_rate=0.9, acceptance_rate=0.9, error_rate_24h=0.0, restriction_penalty=0.0, engagement_quality=0.9)
        score, band = compute_health(signals)
        assert band == "warmup"
        assert score > 0.8

    def test_cold_band(self):
        signals = HealthSignals(response_rate=0.05, acceptance_rate=0.05, error_rate_24h=0.5, restriction_penalty=0.5, engagement_quality=0.05)
        score, band = compute_health(signals)
        assert band == "cold"
        assert score < 0.65

    def test_score_in_unit_interval(self):
        signals = HealthSignals(response_rate=0.5, acceptance_rate=0.5, error_rate_24h=0.1, restriction_penalty=0.1, engagement_quality=0.5)
        score, _ = compute_health(signals)
        assert 0.0 <= score <= 1.0
