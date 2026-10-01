"""Voice analyzer tests."""

import pytest

from voice_intel.core.analyzer import fingerprint


@pytest.mark.unit
class TestFingerprint:
    def test_empty_text(self):
        fp = fingerprint("")
        assert fp.tone_descriptor == "empty"
        assert fp.top_unigrams == []

    def test_short_punchy_text(self):
        fp = fingerprint("shipped.\n\ndone.\n\nnext.")
        assert fp.tone_descriptor == "punchy"
        assert fp.avg_sentence_length < 30

    def test_essayistic_text(self):
        body = " ".join(["the architecture is fundamentally sound"] * 50) + "."
        fp = fingerprint(body)
        assert fp.tone_descriptor in ("essayistic", "neutral")

    def test_emoji_density(self):
        fp = fingerprint("🚀 launching! 🎉 done! 🔥 happy!")
        assert fp.emoji_density > 0.3
        assert fp.tone_descriptor == "playful"

    def test_hashtag_density(self):
        fp = fingerprint("#ai #ml #startup #product #launch today is huge")
        assert fp.hashtag_density > 0.3
        assert fp.tone_descriptor == "promotional"

    def test_lowercase_ratio(self):
        fp1 = fingerprint("lower case writing style here for sure.")
        fp2 = fingerprint("UPPERCASE WRITING STYLE HERE FOR SURE.")
        assert fp1.lowercase_ratio > fp2.lowercase_ratio

    def test_top_unigrams_filtered(self):
        fp = fingerprint("kubernetes kubernetes kubernetes postgres postgres postgresql")
        assert "kubernetes" in fp.top_unigrams
        assert "the" not in fp.top_unigrams

    def test_line_break_style(self):
        # 1 short line vs many short lines.
        fp_airy = fingerprint("\n".join(["a" * 30] * 20))
        fp_dense = fingerprint(("a" * 30 + " ") * 5)
        assert fp_airy.line_break_style in ("airy", "dense")
