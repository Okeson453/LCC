"""RAG retriever — top-K KB fact retrieval."""

from __future__ import annotations

import asyncio
import os
from typing import Any

from kb_client import KBCategory, KBRecord, KBRetriever
from vector_db import QdrantClientWrapper


_retriever: KBRetriever | None = None


def get_retriever() -> KBRetriever:
    """Lazy-init the global retriever."""
    global _retriever
    if _retriever is None:
        url = os.environ.get("VECTOR_DB_URL", "http://qdrant:6333")
        _retriever = KBRetriever(QdrantClientWrapper(url=url))
    return _retriever


async def retrieve_top_k(
    member_id: str,
    query_embedding: list[float],
    top_k: int = 5,
    category_filter: list[KBCategory] | None = None,
) -> list[KBRecord]:
    """Return up to `top_k` KB records ordered by relevance."""
    retriever = get_retriever()
    try:
        return await retriever.search(
            member_id=member_id,
            query_embedding=query_embedding,
            category_filter=category_filter,
            top_k=top_k,
            min_relevance=0.0,
        )
    except Exception:
        return []  # vector store failure → empty grounding (caller emits low_grounding)
