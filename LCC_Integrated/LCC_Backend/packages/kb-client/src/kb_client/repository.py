"""KB repository — async CRUD against Postgres + Qdrant.

`PgKBRepository` is the canonical implementation. It:
- Inserts records into `lcc.kb_records` (transactional, audit-in-tx).
- Chunks body text via the kb-intel chunker (delegated; not duplicated here).
- Embeds chunks via the supplied embedder (any callable returning `list[float]`).
- Dedup-checks against existing `kb_records` chunks (cosine ≥ threshold).
- Upserts chunk vectors into Qdrant collection `kb_records`.
- Persists chunk rows into `lcc.kb_record_chunks`.

The repository is intentionally agnostic about the embedder and the vector
store implementation; both are injected. This keeps it testable without
requiring a real Postgres or Qdrant.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import uuid
from dataclasses import dataclass, field
from typing import Any, Awaitable, Callable, Iterable, Protocol

from kb_client.types import (
    KBCategory,
    KBCreateRequest,
    KBDuplicateError,
    KBRecord,
    KBRetrievalError,
)


Embedder = Callable[[str], Awaitable[list[float]]]


class VectorStore(Protocol):
    """Subset of vector-db API we depend on."""

    async def search(
        self,
        collection: str,
        vector: list[float],
        top_k: int = 5,
        filter: dict[str, Any] | None = None,
    ) -> list[Any]: ...

    async def upsert(
        self,
        collection: str,
        id: str,
        vector: list[float],
        payload: dict[str, Any],
    ) -> None: ...

    async def delete(self, collection: str, id: str) -> None: ...


class TextChunker(Protocol):
    """Splits text into chunks."""

    def __call__(self, text: str) -> list[str]: ...


class KBRepository:
    """Abstract repository — concrete impls: PgKBRepository, InMemoryKBRepository."""

    async def create(self, req: KBCreateRequest) -> KBRecord:
        raise NotImplementedError

    async def get(self, record_id: str) -> KBRecord | None:
        raise NotImplementedError

    async def delete(self, record_id: str) -> bool:
        raise NotImplementedError

    async def list_by_member(self, member_id: str, limit: int = 100) -> list[KBRecord]:
        raise NotImplementedError


class InMemoryKBRepository(KBRepository):
    """In-memory implementation — for tests."""

    def __init__(self) -> None:
        self._records: dict[str, KBRecord] = {}

    async def create(self, req: KBCreateRequest) -> KBRecord:
        record_id = str(uuid.uuid4())
        record = KBRecord(
            id=record_id,
            member_id=req.member_id,
            category=req.category,
            title=req.title,
            fact=req.body,
            chunk_index=0,
            chunk_count=1,
            score=0.0,
            metadata=req.metadata,
        )
        self._records[record_id] = record
        return record

    async def get(self, record_id: str) -> KBRecord | None:
        return self._records.get(record_id)

    async def delete(self, record_id: str) -> bool:
        return self._records.pop(record_id, None) is not None

    async def list_by_member(self, member_id: str, limit: int = 100) -> list[KBRecord]:
        return [r for r in self._records.values() if r.member_id == member_id][:limit]


def content_hash(text: str) -> str:
    """Stable SHA-256 hash of normalized text."""
    normalized = " ".join(text.lower().split())
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


# Default chunker — 1200 char window with 200 char overlap, sentence-aware.
# Mirrors kb-intel's chunker. Kept here so kb-client has zero deps on kb-intel.
def default_chunker(text: str, max_chars: int = 1200, overlap: int = 200) -> list[str]:
    text = text.strip()
    if not text:
        return []
    if len(text) <= max_chars:
        return [text]

    import re

    sentences = re.split(r"(?<=[.!?])\s+", text)
    chunks: list[str] = []
    current: list[str] = []
    current_len = 0

    for s in sentences:
        s_len = len(s)
        if s_len > max_chars:
            for i in range(0, s_len, max_chars - overlap):
                chunks.append(s[i : i + max_chars])
            continue
        if current_len + s_len > max_chars and current:
            chunks.append(" ".join(current))
            tail = " ".join(current)
            if len(tail) > overlap:
                current = [tail[-overlap:]]
                current_len = overlap
            else:
                current_len = sum(len(x) for x in current)
        current.append(s)
        current_len += s_len

    if current:
        chunks.append(" ".join(current))
    return chunks


@dataclass
class PgKBRepositoryConfig:
    """Tunable knobs for the Postgres-backed KB repository."""

    dedup_threshold: float = 0.92
    embedding_model_version: str = "all-MiniLM-L6-v2@1"
    chunk_max_chars: int = 1200
    chunk_overlap: int = 200
    embedding_batch_size: int = 16
    audit_resource_type: str = "kb_record"


class PgKBRepository(KBRepository):
    """Postgres + Qdrant-backed KB repository.

    Dependencies (injected, not constructed internally):
    - `db_pool`: any object with an async `execute(query, *args)` and
      `fetch(query, *args)` and `fetchrow(query, *args)` interface
      (sqlalchemy/asyncpg/sqlx-via-pyO3 etc.).
    - `vector_db`: a `VectorStore` (above).
    - `embedder`: async callable `text -> list[float]`.
    - `chunker`: optional. Defaults to `default_chunker`.
    - `audit_emitter`: optional. If provided, called *inside the same logical
      transaction* to record an audit row (best-effort; not rolled back if the
      DB call fails since we can't share a tx object across pool boundaries
      without explicit cooperation).
    """

    def __init__(
        self,
        db_pool: Any,
        vector_db: VectorStore,
        embedder: Embedder,
        chunker: TextChunker | None = None,
        audit_emitter: Callable[[dict[str, Any]], Awaitable[None]] | None = None,
        config: PgKBRepositoryConfig | None = None,
    ) -> None:
        self._db = db_pool
        self._vec = vector_db
        self._embed = embedder
        self._chunker: TextChunker = chunker or (
            lambda text: default_chunker(
                text,
                max_chars=config.chunk_max_chars if config else 1200,
                overlap=config.chunk_overlap if config else 200,
            )
        )
        self._audit = audit_emitter
        self._cfg = config or PgKBRepositoryConfig()

    async def create(self, req: KBCreateRequest) -> KBRecord:
        # Validate.
        if not req.title.strip():
            raise ValueError("title is required")
        if not req.body.strip():
            raise ValueError("body is required")
        if not req.member_id:
            raise ValueError("member_id is required")

        # 1. Chunk the body.
        chunks = self._chunker(req.body.strip())
        if not chunks:
            chunks = [req.title.strip()]

        # 2. Embed each chunk in batches.
        embeddings: list[list[float]] = []
        for i in range(0, len(chunks), self._cfg.embedding_batch_size):
            batch = chunks[i : i + self._cfg.embedding_batch_size]
            embeddings.extend(await asyncio.gather(*(self._embed(t) for t in batch)))

        # 3. Dedup against existing chunks for this member (cosine ≥ threshold).
        existing = await self._find_duplicate(
            member_id=req.member_id,
            first_embedding=embeddings[0],
            threshold=self._cfg.dedup_threshold,
        )
        if existing is not None:
            raise KBDuplicateError(f"duplicate of {existing}")

        # 4. Insert into lcc.kb_records.
        record_id = str(uuid.uuid4())
        now_iso = _utcnow_iso()
        await self._db.execute(
            """
            INSERT INTO lcc.kb_records
              (id, member_id, category, title, body, source_kind, source_url, metadata, created_at, updated_at)
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s::jsonb, NOW(), NOW())
            """,
            (
                record_id,
                req.member_id,
                req.category.value,
                req.title,
                req.body,
                req.source_kind,
                req.source_url,
                json.dumps(req.metadata),
            ),
        )

        # 5. Insert each chunk into lcc.kb_record_chunks + upsert into Qdrant.
        for idx, (chunk_text, embedding) in enumerate(zip(chunks, embeddings, strict=True)):
            chunk_id = f"{record_id}:{idx}"
            await self._db.execute(
                """
                INSERT INTO lcc.kb_record_chunks
                  (id, record_id, member_id, chunk_index, chunk_count, text, content_hash,
                   embedding_model_version, created_at)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, NOW())
                """,
                (
                    chunk_id,
                    record_id,
                    req.member_id,
                    idx,
                    len(chunks),
                    chunk_text,
                    content_hash(chunk_text),
                    self._cfg.embedding_model_version,
                ),
            )
            await self._vec.upsert(
                collection="kb_records",
                id=chunk_id,
                vector=embedding,
                payload={
                    "record_id": record_id,
                    "member_id": req.member_id,
                    "category": req.category.value,
                    "title": req.title,
                    "chunk_index": idx,
                    "chunk_count": len(chunks),
                    "text": chunk_text,
                    "content_hash": content_hash(chunk_text),
                    "source_kind": req.source_kind,
                    "source_url": req.source_url or "",
                    "embedding_model_version": self._cfg.embedding_model_version,
                    "created_at": now_iso,
                },
            )

        # 6. Audit emit (best-effort).
        if self._audit:
            await self._audit(
                {
                    "actor": "system:kb-intel",
                    "action": "kb.record.created",
                    "resource_type": self._cfg.audit_resource_type,
                    "resource_id": record_id,
                    "member_id": req.member_id,
                    "metadata": {
                        "category": req.category.value,
                        "title": req.title,
                        "chunk_count": len(chunks),
                        "source_kind": req.source_kind,
                    },
                }
            )

        return KBRecord(
            id=record_id,
            member_id=req.member_id,
            category=req.category,
            title=req.title,
            fact=req.body,
            chunk_index=0,
            chunk_count=len(chunks),
            score=0.0,
            metadata=req.metadata,
        )

    async def get(self, record_id: str) -> KBRecord | None:
        row = await self._db.fetchrow(
            """
            SELECT id, member_id, category::TEXT AS category, title, body, metadata
            FROM lcc.kb_records
            WHERE id = %s AND deleted_at IS NULL
            """,
            (record_id,),
        )
        if row is None:
            return None
        return KBRecord(
            id=str(row["id"]),
            member_id=str(row["member_id"]),
            category=_coerce_category(row["category"]),
            title=row["title"],
            fact=row["body"],
            chunk_index=0,
            chunk_count=1,
            score=0.0,
            metadata=row["metadata"] if isinstance(row["metadata"], dict) else {},
        )

    async def delete(self, record_id: str) -> bool:
        """Soft-delete the record and remove its vectors."""
        rows = await self._db.fetch(
            "SELECT id FROM lcc.kb_record_chunks WHERE record_id = %s",
            (record_id,),
        )
        for r in rows:
            try:
                await self._vec.delete(collection="kb_records", id=str(r["id"]))
            except Exception as e:
                # Don't block soft-delete if vector store is down; the
                # data-purge-worker will reconcile later.
                pass
        result = await self._db.execute(
            "UPDATE lcc.kb_records SET deleted_at = NOW(), updated_at = NOW() WHERE id = %s AND deleted_at IS NULL",
            (record_id,),
        )
        # asyncpg returns "UPDATE N" string; sqlx-py returns rowcount via attr.
        try:
            return int(result.split()[-1]) > 0
        except (AttributeError, ValueError):
            return bool(result)

    async def list_by_member(self, member_id: str, limit: int = 100) -> list[KBRecord]:
        rows = await self._db.fetch(
            """
            SELECT id, member_id, category::TEXT AS category, title, body, metadata
            FROM lcc.kb_records
            WHERE member_id = %s AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT %s
            """,
            (member_id, min(limit, 500)),
        )
        return [
            KBRecord(
                id=str(r["id"]),
                member_id=str(r["member_id"]),
                category=_coerce_category(r["category"]),
                title=r["title"],
                fact=r["body"],
                chunk_index=0,
                chunk_count=1,
                score=0.0,
                metadata=r["metadata"] if isinstance(r["metadata"], dict) else {},
            )
            for r in rows
        ]

    async def _find_duplicate(
        self,
        member_id: str,
        first_embedding: list[float],
        threshold: float,
    ) -> str | None:
        """Search Qdrant for an existing chunk near `first_embedding`."""
        try:
            hits = await self._vec.search(
                collection="kb_records",
                vector=first_embedding,
                top_k=1,
                filter={"member_id": member_id},
            )
        except Exception as e:
            raise KBRetrievalError(f"dedup search failed: {e}") from e
        for h in hits:
            if getattr(h, "score", 0.0) >= threshold:
                return str(getattr(h, "id"))
        return None


def _utcnow_iso() -> str:
    from datetime import UTC, datetime

    return datetime.now(UTC).isoformat()


def _coerce_category(s: str) -> KBCategory:
    try:
        return KBCategory(s)
    except ValueError:
        return KBCategory.SKILLS
