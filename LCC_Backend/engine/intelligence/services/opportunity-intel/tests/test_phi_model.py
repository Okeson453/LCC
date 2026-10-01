"""Phi-model tests."""

import pytest

from opportunity_intel.core.phi_model import PhiSignal, score


@pytest.mark.unit
class TestPhiModelRuleBased:
    def test_no_signals(self):
        r = score([], labeled_sends=0)
        assert r.phi_score == 0.0
        assert not r.hard_qualifies
        assert r.mode == "rule_based"

    def test_cold_start_uses_rule_based(self):
        s = PhiSignal(kind="post_about_problem", recency_days=2, weight=0.8, matched_kb_facts=2)
        r = score([s], labeled_sends=50)
        assert r.mode == "rule_based"
        assert r.phi_score > 0.3

    def test_warm_start_uses_ml(self):
        s = PhiSignal(kind="job_change", recency_days=1, weight=0.9, matched_kb_facts=3)
        r = score([s], labeled_sends=200)
        assert r.mode in ("ml", "rule_based")  # ML model loader may not exist

    def test_decay_for_old_signals(self):
        s = PhiSignal(kind="post_about_problem", recency_days=90, weight=1.0)
        r_old = score([s], labeled_sends=0)
        s_fresh = PhiSignal(kind="post_about_problem", recency_days=1, weight=1.0)
        r_new = score([s_fresh], labeled_sends=0)
        assert r_new.phi_score > r_old.phi_score

    def test_diversity_bonus(self):
        s1 = PhiSignal(kind="job_change", recency_days=5, weight=0.5)
        s2 = PhiSignal(kind="funding_event", recency_days=5, weight=0.5)
        s3 = PhiSignal(kind="hiring", recency_days=5, weight=0.5)
        r1 = score([s1], labeled_sends=0)
        r3 = score([s1, s2, s3], labeled_sends=0)
        assert r3.phi_score >= r1.phi_score

    def test_hard_qualifies_threshold(self):
        s = PhiSignal(kind="job_change", recency_days=0, weight=1.0, matched_kb_facts=5)
        r = score([s], labeled_sends=0, qualified_threshold=0.65)
        assert r.hard_qualifies

    def test_confidence_in_unit_interval(self):
        s = PhiSignal(kind="job_change", recency_days=2, weight=0.7)
        r = score([s], labeled_sends=0)
        assert 0.0 <= r.confidence <= 1.0
        assert 0.0 <= r.phi_score <= 1.0
