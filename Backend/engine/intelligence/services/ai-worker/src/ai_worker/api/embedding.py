"""Embedding endpoint."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict

from ai_worker.infra.embedding import embed_query

router = APIRouter()


class EmbedRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    text: str
    model_hint: str | None = None


class EmbedResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    embedding: list[float]
    model_id: str
    latency_ms: int


@router.post("/embed", response_model=EmbedResponse)
async def embed(req: EmbedRequest) -> EmbedResponse:
    embedding, model_id, latency = await embed_query(req.text, model_hint=req.model_hint)
    return EmbedResponse(embedding=embedding, model_id=model_id, latency_ms=latency)
