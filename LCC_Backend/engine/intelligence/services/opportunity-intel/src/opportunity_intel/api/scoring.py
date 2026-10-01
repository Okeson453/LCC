"""φ scoring endpoint."""

from __future__ import annotations

from typing import Literal

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, ConfigDict, Field

from opportunity_intel.core.phi_model import PhiResult, PhiSignal, score

router = APIRouter()


class PhiSignalRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    kind: Literal[
        "job_change", "funding_event", "post_about_problem",
        "competitor_mention", "manual", "company_growth", "hiring",
    ]
    recency_days: int = Field(ge=0, le=180)
    weight: float = Field(ge=0, le=1)
    text: str = ""
    matched_kb_facts: int = 0


class PhiScoreRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    opportunity_id: str | None = None
    signals: list[PhiSignalRequest]
    labeled_sends: int = 0
    qualified_threshold: float = 0.65


class PhiScoreResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    phi_score: float
    components: dict[str, float]
    mode: str
    confidence: float
    hard_qualifies: bool
    rationale: list[str]
    trace_id: str


@router.post("/phi/score", response_model=PhiScoreResponse)
async def phi_score(req: PhiScoreRequest) -> PhiScoreResponse:
    signals = [
        PhiSignal(
            kind=s.kind,
            recency_days=s.recency_days,
            weight=s.weight,
            text=s.text,
            matched_kb_facts=s.matched_kb_facts,
        )
        for s in req.signals
    ]
    if not signals:
        raise HTTPException(status_code=400, detail="signals required")

    result: PhiResult = score(
        signals=signals,
        labeled_sends=req.labeled_sends,
        qualified_threshold=req.qualified_threshold,
    )
    return PhiScoreResponse(
        phi_score=result.phi_score,
        components=result.components,
        mode=result.mode,
        confidence=result.confidence,
        hard_qualifies=result.hard_qualifies,
        rationale=result.rationale,
        trace_id=f"phi:{req.opportunity_id or 'n/a'}",
    )
