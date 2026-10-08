"""KB ingest endpoint — creates a new KB record with chunking + dedup."""

from __future__ import annotations

from typing import Literal

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, ConfigDict, Field

from kb_intel.core.chunker import chunk as chunk_text
from kb_intel.core.dedup import content_hash, find_duplicate
from kb_intel.core.embedder import embed_texts
from vector_db import QdrantClientWrapper

router = APIRouter()


class IngestRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    category: Literal[
        "achievements",
        "skills",
        "projects",
        "experience",
        "voice_samples",
        "case_studies",
        "testimonials",
        "preferences",
    ]
    title: str
    body: str = ""
    source_url: str | None = None
    source_kind: Literal["manual", "linkedin", "document_upload", "external_import"] = "manual"
    duplicate_threshold: float = 0.92


class IngestResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    record_id: str
    chunks: int
    dedup_hit: str | None = None
    created_at: str


@router.post("/ingest", response_model=IngestResponse)
async def ingest(req: IngestRequest) -> IngestResponse:
    import os
    import uuid
    from datetime import UTC, datetime

    text = (req.title + "\n\n" + (req.body or "")).strip()
    if not text:
        raise HTTPException(status_code=400, detail="empty kb record")

    chunks = chunk_text(text, max_chars=int(os.environ.get("KB_INTEL_CHUNK_MAX_CHARS", "1200")))
    if not chunks:
        raise HTTPException(status_code=400, detail="empty chunks")

    db = QdrantClientWrapper(url=os.environ.get("VECTOR_DB_URL", "http://qdrant:6333"))
    embeddings = await embed_texts(chunks)

    # Dedup: check first chunk vs existing.
    dedup_hit = await find_duplicate(
        db=db,
        member_id=req.member_id,
        embedding=embeddings[0],
        threshold=req.duplicate_threshold,
    )
    if dedup_hit:
        return IngestResponse(
            record_id=dedup_hit,
            chunks=len(chunks),
            dedup_hit=dedup_hit,
            created_at=datetime.now(UTC).isoformat(),
        )

    record_id = str(uuid.uuid4())
    now = datetime.now(UTC).isoformat()
    # F-AUDIT-86: this loop variable was named `chunk_text`, shadowing the
    # `from kb_intel.core.chunker import chunk as chunk_text` above. Because
    # Python treats `chunk_text` as a local for the whole function, the earlier
    # call
    #     chunks = chunk_text(text, ...)
    # raised `UnboundLocalError: cannot access local variable 'chunk_text'`
    # at runtime — so this endpoint failed on every non-empty record. Ruff
    # reported it as F823 (referenced before assignment) and F402 (import
    # shadowed by loop variable).
    for i, (chunk, embedding) in enumerate(zip(chunks, embeddings, strict=False)):
        chunk_id = f"{record_id}:{i}"
        await db.upsert(
            collection="kb_records",
            id=chunk_id,
            vector=embedding,
            payload={
                "record_id": record_id,
                "member_id": req.member_id,
                "category": req.category,
                "title": req.title,
                "chunk_index": i,
                "chunk_count": len(chunks),
                "text": chunk,
                "content_hash": content_hash(chunk),
                "source_kind": req.source_kind,
                "source_url": req.source_url or "",
                "created_at": now,
            },
        )

    return IngestResponse(record_id=record_id, chunks=len(chunks), created_at=now)
