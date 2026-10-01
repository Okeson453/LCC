"""Discovery endpoint — finds candidate opportunities and triggers enrichment."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict, Field

from opportunity_intel.core.phi_model import PhiSignal, score

router = APIRouter()


class DiscoveryRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    lookback_days: int = 14
    min_signal_score: float = 0.4
    labeled_sends: int = 0
    trace_id: str | None = None


class DiscoveredOpportunity(BaseModel):
    model_config = ConfigDict(extra="forbid")
    contact_id: str
    kind: str
    phi_score: float
    rationale: list[str]


class DiscoveryResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    member_id: str
    opportunities: list[DiscoveredOpportunity]
    mode: str
    trace_id: str


# --- Mock discovery backend ---
# In production this is fed by the LinkedIn data broker via enrichment_worker
# (which fetches ProfileSnapshot, PostSnapshot, JobSnapshot from REST + jobs APIs).
# The service compiles signal observations and forwards to phi_model.score.

@router.post("/discover", response_model=DiscoveryResponse)
async def discover(req: DiscoveryRequest) -> DiscoveryResponse:
    """Stub discovery: produce 0..N candidate opportunities from cached signals.

    In production, this would call the network_crm_svc to fetch recent
    contact signals and rank them.
    """
    mock_signals = [
        # Two example signals per member — empty by default.
    ]
    if req.member_id and not mock_signals:
        # No real data: return empty list.
        return DiscoveryResponse(
            member_id=req.member_id,
            opportunities=[],
            mode="idle",
            trace_id=req.trace_id or f"disc:{req.member_id}",
        )

    opportunities: list[DiscoveredOpportunity] = []
    for i, sig in enumerate(mock_signals):
        phi = score(
            signals=[sig],
            labeled_sends=req.labeled_sends,
            qualified_threshold=0.65,
        )
        if phi.phi_score >= req.min_signal_score:
            opportunities.append(
                DiscoveredOpportunity(
                    contact_id=f"contact-{i}",
                    kind=sig.kind,
                    phi_score=phi.phi_score,
                    rationale=phi.rationale,
                )
            )
    return DiscoveryResponse(
        member_id=req.member_id,
        opportunities=opportunities,
        mode="rule_based" if req.labeled_sends < 200 else "ml",
        trace_id=req.trace_id or f"disc:{req.member_id}",
    )
