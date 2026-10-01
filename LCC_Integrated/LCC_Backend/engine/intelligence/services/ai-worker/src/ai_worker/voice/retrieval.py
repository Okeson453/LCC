"""Voice sample retriever — top-K past approved posts for tone matching."""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any

from vector_db import QdrantClientWrapper


@dataclass
class VoiceSampleHit:
    text: str
    score: float


_voice_db: QdrantClientWrapper | None = None


def get_voice_db() -> QdrantClientWrapper:
    global _voice_db
    if _voice_db is None:
        url = os.environ.get("VECTOR_DB_URL", "http://qdrant:6333")
        _voice_db = QdrantClientWrapper(url=url)
    return _voice_db


async def retrieve_voice_samples(
    member_id: str, query_embedding: list[float], top_k: int = 3
) -> list[VoiceSampleHit]:
    """Return top-K voice samples matching the query embedding."""
    db = get_voice_db()
    try:
        hits = await db.search(
            collection="voice_samples",
            vector=query_embedding,
            top_k=top_k,
            filter={"member_id": member_id},
        )
        return [
            VoiceSampleHit(
                text=str(hit.payload.get("text", "")),
                score=hit.score,
            )
            for hit in hits
            if hit.payload.get("text")
        ]
    except Exception:
        return []


# Convenience re-export for tests.
__all__ = ["VoiceSampleHit", "retrieve_voice_samples"]
