"""KB client tests."""

import pytest

from kb_client.base import KBCategory, KBRecord, KBClientError
from kb_client.retriever import KBRetriever


@pytest.mark.unit
class TestKBRecord:
    def test_record_creation(self):
        record = KBRecord(
            id="rec-1",
            member_id="m-1",
            category=KBCategory.ACHIEVEMENTS,
            title="Shipped v2",
            fact="Shipped the v2 launch on time with 5% latency reduction.",
            chunk_index=0,
            chunk_count=2,
            score=0.92,
        )
        assert record.id == "rec-1"
        assert record.category == KBCategory.ACHIEVEMENTS

    def test_record_metadata(self):
        record = KBRecord(
            id="rec-2",
            member_id="m-1",
            category=KBCategory.SKILLS,
            title="Rust",
            fact="10 years of Rust.",
            chunk_index=0,
            chunk_count=1,
            score=0.8,
            metadata={"source_kind": "manual"},
        )
        assert record.metadata["source_kind"] == "manual"


@pytest.mark.unit
class TestKBCategory:
    def test_categories_distinct(self):
        assert KBCategory.ACHIEVEMENTS != KBCategory.SKILLS
        assert KBCategory.VOICE_SAMPLES != KBCategory.PREFERENCES

    def test_category_string_value(self):
        assert KBCategory.ACHIEVEMENTS.value == "achievements"
        assert KBCategory.CASE_STUDIES.value == "case_studies"
        assert KBCategory.TESTIMONIALS.value == "testimonials"


@pytest.mark.unit
class TestKBClientErrors:
    def test_kb_client_error_is_exception(self):
        assert issubclass(KBClientError, Exception)
        err = KBClientError("kaboom")
        assert str(err) == "kaboom"


@pytest.mark.unit
class TestKBRetriever:
    def test_retriever_init(self):
        from vector_db import QdrantClientWrapper
        retriever = KBRetriever(QdrantClientWrapper())
        assert retriever is not None
