"""Fingerprint endpoints."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict, Field

from voice_intel.core.analyzer import fingerprint

router = APIRouter()


class FingerprintRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    text: str = ""
    samples: list[str] = Field(default_factory=list)


class FingerprintResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    fingerprint: dict
    sample_count: int


@router.post("/fingerprint", response_model=FingerprintResponse)
async def compute_fingerprint(req: FingerprintRequest) -> FingerprintResponse:
    samples = list(req.samples)
    if req.text and not samples:
        samples = [req.text]
    text = "\n\n".join(samples).strip()
    fp = fingerprint(text)
    return FingerprintResponse(fingerprint=fp.to_dict(), sample_count=len(samples))
