"""Vector DB tests — base interface + error types."""

import pytest

from vector_db.base import SearchHit, VectorDB, VectorDBError, VectorStoreUnavailable


@pytest.mark.unit
class TestVectorDBError:
    def test_error_is_exception(self):
        assert issubclass(VectorDBError, Exception)
        err = VectorDBError("kaboom")
        assert str(err) == "kaboom"

    def test_unavailable_error_is_vector_error(self):
        err = VectorStoreUnavailable("down")
        assert err.args == ("down",)
        assert isinstance(err, VectorDBError)


@pytest.mark.unit
class TestSearchHit:
    def test_search_hit_creation(self):
        hit = SearchHit(id="h1", score=0.92, payload={"text": "hi"})
        assert hit.id == "h1"
        assert hit.score == 0.92
        assert hit.payload["text"] == "hi"


@pytest.mark.unit
class TestVectorDBInterface:
    def test_interface_methods_exist(self):
        # VectorDB is a Protocol; assert required methods are defined.
        for method in ("search", "upsert", "delete", "ensure_collection"):
            assert hasattr(VectorDB, method), f"missing {method}"
