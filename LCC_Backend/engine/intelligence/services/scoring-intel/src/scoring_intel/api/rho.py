"""ρ scoring endpoint."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict, Field

from scoring_intel.core.rho import ContactFeature, score_contact

router = APIRouter()


class RhoScoreRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    contact_id: str
    is_vip: bool = False
    is_mutual: bool = False
    last_contact_days: int = 365
    matched_kb_facts: int = 0
    response_rate: float = 0.0
    accept_rate: float = 0.0
    labeled_sends: int = 0


class RhoScoreResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    contact_id: str
    score: float
    components: dict[str, float]
    mode: str
    rationale: list[str]


@router.post("/rho/score", response_model=RhoScoreResponse)
async def rho_score(req: RhoScoreRequest) -> RhoScoreResponse:
    contact = ContactFeature(
        contact_id=req.contact_id,
        is_vip=req.is_vip,
        is_mutual=req.is_mutual,
        last_contact_days=req.last_contact_days,
        matched_kb_facts=req.matched_kb_facts,
        response_rate=req.response_rate,
        accept_rate=req.accept_rate,
    )
    result = score_contact(contact, labeled_sends=req.labeled_sends)
    return RhoScoreResponse(
        contact_id=result.contact_id,
        score=result.score,
        components=result.components,
        mode=result.mode,
        rationale=result.rationale,
    )
