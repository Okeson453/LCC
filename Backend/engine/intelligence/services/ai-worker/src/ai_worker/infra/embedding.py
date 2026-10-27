"""Embedding generation — sentence-transformers wrapper."""

from __future__ import annotations

import os
import tempfile
import time
from pathlib import Path

# Lazy import to avoid loading the model on startup in environments where
# vector DB isn't reachable.
_model = None
_DEFAULT_CACHE = Path(tempfile.gettempdir()) / "sbert-cache"


def _get_model():
    global _model
    if _model is None:
        from sentence_transformers import SentenceTransformer

        _model = SentenceTransformer(
            os.environ.get("EMBEDDING_MODEL", "all-MiniLM-L6-v2"),
            cache_folder=os.environ.get("SENTENCE_TRANSFORMERS_HOME", str(_DEFAULT_CACHE)),
        )
    return _model


def _dim() -> int:
    model = _get_model()
    return int(model.get_sentence_embedding_dimension())


async def embed_query(text: str, model_hint: str | None = None) -> tuple[list[float], str, int]:
    """Embed a single text. Returns (embedding, model_id, latency_ms)."""
    start = time.monotonic()
    model = _get_model()
    embedding = model.encode(text, convert_to_numpy=True).tolist()
    latency = int((time.monotonic() - start) * 1000)
    model_id = model_hint or os.environ.get("EMBEDDING_MODEL", "all-MiniLM-L6-v2")
    return embedding, model_id, latency


async def embed_batch(texts: list[str]) -> list[list[float]]:
    """Embed multiple texts in a single batch (more efficient)."""
    model = _get_model()
    return [e.tolist() for e in model.encode(texts, convert_to_numpy=True)]


def embedding_dim() -> int:
    return _dim()
