"""Qdrant implementation of VectorDB."""

from __future__ import annotations

import asyncio
from typing import Any

from qdrant_client import AsyncQdrantClient
from qdrant_client.http import models as qmodels
from qdrant_client.http.exceptions import UnexpectedResponse

from vector_db.base import SearchHit, VectorDB, VectorDBError, VectorStoreUnavailable


class QdrantClientWrapper(VectorDB):
    """Async wrapper around the official qdrant-client.

    Connection settings are configured via the constructor; the wrapper
    exposes the LCC-required surface.
    """

    def __init__(self, url: str = "http://localhost:6333", api_key: str | None = None) -> None:
        self._client = AsyncQdrantClient(url=url, api_key=api_key)

    async def search(
        self,
        collection: str,
        vector: list[float],
        top_k: int = 5,
        filter: dict[str, Any] | None = None,
    ) -> list[SearchHit]:
        try:
            qfilter = self._build_filter(filter or {})
            results = await self._client.search(
                collection_name=collection,
                query_vector=vector,
                limit=top_k,
                query_filter=qfilter,
                with_payload=True,
            )
        except (UnexpectedResponse, ConnectionError, TimeoutError) as e:
            raise VectorStoreUnavailable(f"qdrant search failed: {e}") from e
        except Exception as e:  # noqa: BLE001
            raise VectorDBError(f"qdrant search failed: {e}") from e

        return [
            SearchHit(
                id=str(h.id),
                score=float(h.score),
                payload=dict(h.payload or {}),
            )
            for h in results
        ]

    async def upsert(
        self,
        collection: str,
        id: str,
        vector: list[float],
        payload: dict[str, Any],
    ) -> None:
        try:
            await self._client.upsert(
                collection_name=collection,
                points=[
                    qmodels.PointStruct(id=id, vector=vector, payload=payload),
                ],
            )
        except Exception as e:  # noqa: BLE001
            raise VectorDBError(f"qdrant upsert failed: {e}") from e

    async def delete(self, collection: str, id: str) -> None:
        try:
            await self._client.delete(
                collection_name=collection,
                points_selector=qmodels.PointIdsList(points=[id]),
            )
        except Exception as e:  # noqa: BLE001
            raise VectorDBError(f"qdrant delete failed: {e}") from e

    async def ensure_collection(
        self,
        collection: str,
        vector_dim: int,
        distance: str = "cosine",
    ) -> None:
        try:
            existing = await self._client.get_collections()
            if any(c.name == collection for c in existing.collections):
                return
        except Exception:  # noqa: BLE001
            # If get_collections fails, try to create and ignore errors.
            pass

        dist = {
            "cosine": qmodels.Distance.COSINE,
            "euclid": qmodels.Distance.EUCLID,
            "dot": qmodels.Distance.DOT,
        }.get(distance.lower(), qmodels.Distance.COSINE)

        try:
            await self._client.create_collection(
                collection_name=collection,
                vectors_config=qmodels.VectorParams(size=vector_dim, distance=dist),
            )
        except UnexpectedResponse as e:
            if "already exists" in str(e).lower():
                return
            raise VectorDBError(f"qdrant create_collection failed: {e}") from e

    @staticmethod
    def _build_filter(f: dict[str, Any]) -> qmodels.Filter | None:
        if not f:
            return None
        must = [
            qmodels.FieldCondition(
                key=k,
                match=qmodels.MatchValue(value=v),
            )
            for k, v in f.items()
        ]
        return qmodels.Filter(must=must)
