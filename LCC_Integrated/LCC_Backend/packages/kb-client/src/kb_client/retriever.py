"""KB retriever — top-K vector search with category filters."""

from __future__ import annotations

from typing import Any

from kb_client.types import KBCategory, KBRecord, KBRetrievalError


class KBRetriever:
    """Top-K retrieval over a vector store, optionally filtered by category."""

    def __init__(self, vector_db: Any) -> None:
        self._vec = vector_db

    async def search(
        self,
        member_id: str,
        query_embedding: list[float],
        *,
        top_k: int = 5,
        category_filter: list[KBCategory] | None = None,
        min_relevance: float = 0.0,
    ) -> list[KBRecord]:
        """Return up to `top_k` records ordered by relevance.

        Raises KBRetrievalError on transport failure.
        """
        payload_filter: dict[str, Any] = {"member_id": member_id}
        if category_filter:
            payload_filter["category"] = [c.value for c in category_filter]
        try:
            hits = await self._vec.search(
                collection="kb_records",
                vector=query_embedding,
                top_k=top_k,
                filter=payload_filter,
            )
        except Exception as e:  # noqa: BLE001
            raise KBRetrievalError(f"vector search failed: {e}") from e

        records: list[KBRecord] = []
        for hit in hits:
            if hit.score < min_relevance:
                continue
            payload = hit.payload or {}
            try:
                category = KBCategory(payload.get("category", "skills"))
            except ValueError:
                category = KBCategory.SKILLS
            records.append(
                KBRecord(
                    id=str(hit.id),
                    member_id=payload.get("member_id", member_id),
                    category=category,
                    title=payload.get("title", ""),
                    fact=payload.get("text", ""),
                    chunk_index=payload.get("chunk_index", 0),
                    chunk_count=payload.get("chunk_count", 1),
                    score=hit.score,
                    metadata={
                        k: v
                        for k, v in payload.items()
                        if k not in ("member_id", "category", "title", "text", "chunk_index", "chunk_count")
                    },
                )
            )
        return records
