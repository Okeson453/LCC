"""Brand-guard linter tests."""

import pytest
from ai_worker.brand_guard.linter import lint_draft


@pytest.mark.unit
class TestBrandGuard:
    def test_clean_post_passes(self):
        # A clean draft that also fits the documented text_post length window
        # (200-3000 chars) — the guard checks length, so a two-line fixture
        # fails for the wrong reason.
        body = (
            "shipped the new feature today. the dashboard now streams events instead of polling, "
            "so the numbers update in place instead of every thirty seconds.\n"
            "finally got the latency down to 80ms at p99, and the retry path no longer drops "
            "a burst when one upstream stalls.\n"
            "next up: backpressure, then the migration guide."
        )
        result = lint_draft(body, target="text_post")
        assert result.passed
        assert result.length_ok
        assert not result.banned_phrases_found

    def test_banned_phrase_fails(self):
        body = "in today's fast-paced world, we must leverage synergy to succeed."
        result = lint_draft(body, target="text_post")
        assert not result.passed
        assert len(result.banned_phrases_found) > 0

    def test_fluff_only_passes(self):
        # Single fluff phrase shouldn't fail; multiple do.
        body = "moreover, " * 1 + "a great post."
        result = lint_draft(body, target="text_post")
        # Body too short → length check fails.
        assert not result.length_ok

    def test_length_window_enforced(self):
        # Too short for text_post.
        body = "hi"
        result = lint_draft(body, target="text_post")
        assert not result.length_ok
        assert not result.passed

    def test_reply_length_window(self):
        body = "a" * 1500  # too long for reply
        result = lint_draft(body, target="reply")
        assert not result.length_ok

    def test_ai_disclaimer_caught(self):
        body = "as an AI, I think the best approach is " + ("x" * 250)
        result = lint_draft(body, target="text_post")
        assert "as an AI" in result.banned_phrases_found

    def test_tone_score_for_clean_prose(self):
        body = "shipped today. latency down 30%. next: customer demo on wednesday."
        result = lint_draft(body, target="text_post")
        assert result.tone_alignment_score > 0.4
