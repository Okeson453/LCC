"""Voice-sample ingest endpoint."""

from __future__ import annotations

import os
import uuid
from datetime import UTC, datetime

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, ConfigDict, Field

from vector_db import QdrantClientWrapper
from voice_intel.core.analyzer import fingerprint

router = APIRouter()


class SampleIngestRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    sample_text: str
    source: str = "approved_post"
    source_url: str | None = None


class SampleIngestResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    sample_id: str
    fingerprint: dict
    created_at: str


@router.post("/samples", response_model=SampleIngestResponse)
async def ingest_sample(req: SampleIngestRequest) -> SampleIngestResponse:
    if not req.sample_text.strip():
        raise HTTPException(status_code=400, detail="empty sample")

    # Compute fingerprint.
    fp = fingerprint(req.sample_text)

    # Embed the sample.
    from kb_intel.core.embedder import embed_query  # reuse embedder (same model)

    embedding = await embed_query(req.sample_text)

    # Upsert to voice_samples collection.
    db = QdrantClientWrapper(url=os.environ.get("VECTOR_DB_URL", "http://qdrant:6333"))
    sample_id = str(uuid.uuid4())
    now = datetime.now(UTC).isoformat()
    await db.upsert(
        collection="voice_samples",
        id=sample_id,
        vector=embedding,
        payload={
            "member_id": req.member_id,
            "text": req.sample_text,
            "source": req.source,
            "source_url": req.source_url or "",
            "fingerprint": fp.to_dict(),
            "created_at": now,
        },
    )
    return SampleIngestResponse(sample_id=sample_id, fingerprint=fp.to_dict(), created_at=now)
