"""Embedding wrapper for kb-intel.

Delegates to the same sentence-transformers model used by ai-worker.
For production scale, kb-intel runs in a separate process and is allowed
to use a different model (e.g., larger all-mpnet-base-v2).
"""

from __future__ import annotations

import os
from functools import lru_cache

_model = None


def _get_model():
    global _model
    if _model is None:
        from sentence_transformers import SentenceTransformer

        _model = SentenceTransformer(
            os.environ.get("KB_INTEL_EMBEDDING_MODEL", "all-MiniLM-L6-v2"),
            cache_folder=os.environ.get("SENTENCE_TRANSFORMERS_HOME", "/tmp/sbert-cache"),
        )
    return _model


async def embed_texts(texts: list[str]) -> list[list[float]]:
    """Embed a batch of texts."""
    model = _get_model()
    return [e.tolist() for e in model.encode(texts, convert_to_numpy=True)]


async def embed_query(text: str) -> list[float]:
    """Embed a single text."""
    model = _get_model()
    return model.encode(text, convert_to_numpy=True).tolist()


__all__ = ["embed_texts", "embed_query"]
