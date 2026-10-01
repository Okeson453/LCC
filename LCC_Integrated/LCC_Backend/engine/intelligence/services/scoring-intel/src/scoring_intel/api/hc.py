"""h_c endpoint."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict

from scoring_intel.core.hc import HealthSignals, compute_health

router = APIRouter()


class HealthRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    response_rate: float = 0.0
    acceptance_rate: float = 0.0
    error_rate_24h: float = 0.0
    restriction_penalty: float = 0.0
    engagement_quality: float = 0.0


class HealthResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    h_c_score: float
    band: str


@router.post("/hc/compute", response_model=HealthResponse)
async def hc_compute(req: HealthRequest) -> HealthResponse:
    signals = HealthSignals(
        response_rate=req.response_rate,
        acceptance_rate=req.acceptance_rate,
        error_rate_24h=req.error_rate_24h,
        restriction_penalty=req.restriction_penalty,
        engagement_quality=req.engagement_quality,
    )
    score, band = compute_health(signals)
    return HealthResponse(h_c_score=score, band=band)
