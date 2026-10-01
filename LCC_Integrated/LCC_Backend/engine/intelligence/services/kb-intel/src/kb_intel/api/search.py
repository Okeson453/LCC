"""KB search endpoint."""

from __future__ import annotations

import os
from typing import Literal

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict, Field

from kb_intel.core.embedder import embed_query
from vector_db import QdrantClientWrapper

router = APIRouter()


class KBSearchRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    query: str
    top_k: int = 5
    category: Literal[
        "achievements", "skills", "projects", "experience",
        "voice_samples", "case_studies", "testimonials", "preferences",
    ] | None = None
    min_score: float = 0.0


class KBHit(BaseModel):
    model_config = ConfigDict(extra="forbid")
    chunk_id: str
    record_id: str
    category: str
    title: str
    text: str
    score: float


class KBSearchResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    hits: list[KBHit]


@router.post("/search", response_model=KBSearchResponse)
async def search(req: KBSearchRequest) -> KBSearchResponse:
    embedding = await embed_query(req.query)
    db = QdrantClientWrapper(url=os.environ.get("VECTOR_DB_URL", "http://qdrant:6333"))
    payload_filter: dict = {"member_id": req.member_id}
    if req.category:
        payload_filter["category"] = req.category
    hits = await db.search(
        collection="kb_records",
        vector=embedding,
        top_k=req.top_k,
        filter=payload_filter,
    )
    out: list[KBHit] = []
    for h in hits:
        if h.score < req.min_score:
            continue
        out.append(
            KBHit(
                chunk_id=h.id,
                record_id=h.payload.get("record_id", h.id),
                category=h.payload.get("category", ""),
                title=h.payload.get("title", ""),
                text=h.payload.get("text", ""),
                score=h.score,
            )
        )
    return KBSearchResponse(hits=out)
