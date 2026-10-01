"""Duplicate detection for KB records."""

from __future__ import annotations

import hashlib

from vector_db import QdrantClientWrapper


def content_hash(text: str) -> str:
    """Stable SHA-256 hash of normalized text."""
    normalized = " ".join(text.lower().split())
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


async def find_duplicate(
    db: QdrantClientWrapper,
    member_id: str,
    embedding: list[float],
    *,
    threshold: float = 0.92,
    top_k: int = 3,
) -> str | None:
    """Return the id of the closest existing KB record above `threshold`, or None."""
    hits = await db.search(
        collection="kb_records",
        vector=embedding,
        top_k=top_k,
        filter={"member_id": member_id},
    )
    for hit in hits:
        if hit.score >= threshold:
            return hit.id
    return None


__all__ = ["content_hash", "find_duplicate"]
