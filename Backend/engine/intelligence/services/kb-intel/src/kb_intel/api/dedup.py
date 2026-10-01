"""Standalone dedup-check endpoint."""

from __future__ import annotations

import os

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict

from kb_intel.core.dedup import find_duplicate
from kb_intel.core.embedder import embed_query
from vector_db import QdrantClientWrapper

router = APIRouter()


class DedupRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    text: str
    threshold: float = 0.92


class DedupResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    duplicate_record_id: str | None
    score: float = 0.0


@router.post("/dedup-check", response_model=DedupResponse)
async def dedup_check(req: DedupRequest) -> DedupResponse:
    embedding = await embed_query(req.text)
    db = QdrantClientWrapper(url=os.environ.get("VECTOR_DB_URL", "http://qdrant:6333"))
    hits = await db.search(
        collection="kb_records", vector=embedding, top_k=1, filter={"member_id": req.member_id},
    )
    if not hits:
        return DedupResponse(duplicate_record_id=None)
    if hits[0].score < req.threshold:
        return DedupResponse(duplicate_record_id=None, score=hits[0].score)
    return DedupResponse(duplicate_record_id=hits[0].id, score=hits[0].score)
