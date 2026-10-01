"""VectorDB — abstract protocol."""

from __future__ import annotations

import abc
from dataclasses import dataclass, field
from typing import Any


class VectorDBError(Exception):
    """Generic vector DB error."""


class VectorStoreUnavailable(VectorDBError):
    """Vector store is down or unreachable. Callers should fall back to degraded mode."""


@dataclass
class SearchHit:
    id: str
    score: float
    payload: dict[str, Any] = field(default_factory=dict)


@dataclass
class VectorSearchResult:
    hits: list[SearchHit]


class VectorDB(abc.ABC):
    """Abstract vector DB."""

    @abc.abstractmethod
    async def search(
        self,
        collection: str,
        vector: list[float],
        top_k: int = 5,
        filter: dict[str, Any] | None = None,
    ) -> list[SearchHit]:
        ...

    @abc.abstractmethod
    async def upsert(
        self,
        collection: str,
        id: str,
        vector: list[float],
        payload: dict[str, Any],
    ) -> None:
        ...

    @abc.abstractmethod
    async def delete(self, collection: str, id: str) -> None:
        ...

    @abc.abstractmethod
    async def ensure_collection(
        self,
        collection: str,
        vector_dim: int,
        distance: str = "cosine",
    ) -> None:
        ...
