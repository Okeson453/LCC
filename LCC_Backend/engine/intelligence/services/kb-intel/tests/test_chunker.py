"""Chunker tests."""

import pytest

from kb_intel.core.chunker import chunk


@pytest.mark.unit
class TestChunker:
    def test_short_text_one_chunk(self):
        text = "this is a short fact."
        chunks = chunk(text, max_chars=200)
        assert chunks == ["this is a short fact."]

    def test_long_text_split(self):
        # 5 sentences, each ~80 chars → with max_chars=200 expect ≥3 chunks.
        sentences = ["a" * 80 + "."] * 10
        text = " ".join(sentences)
        chunks = chunk(text, max_chars=200, overlap=40)
        assert len(chunks) >= 3
        assert all(len(c) <= 220 for c in chunks)  # overlap tolerance

    def test_empty_text(self):
        assert chunk("", max_chars=200) == []
        assert chunk("   ", max_chars=200) == []

    def test_overlap_preserved(self):
        text = ("Sentence one is here. " * 30).strip()
        chunks = chunk(text, max_chars=200, overlap=50)
        # Each adjacent pair must overlap on at least 1 sentence.
        for i in range(len(chunks) - 1):
            tail = chunks[i][-50:]
            assert any(w in chunks[i + 1] for w in tail.split() if len(w) > 3)

    def test_huge_single_sentence(self):
        huge = "x" * 5000
        chunks = chunk(huge, max_chars=1200, overlap=200)
        assert len(chunks) >= 4
        assert all(len(c) <= 1200 for c in chunks)
