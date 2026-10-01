"""ab_d endpoint."""

from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel, ConfigDict, Field

from scoring_intel.core.abd import AbdState, effective_daily_cap, update

router = APIRouter()


class AbdUpdateRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")
    current_multiplier: float = 1.0
    acceptance_rate: float | None = None
    error_rate: float | None = None
    restriction_count_24h: int | None = None
    base_daily_cap: int = 30
    floor: float = 0.5
    ceiling: float = 2.0


class AbdUpdateResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")
    new_multiplier: float
    effective_daily_cap: int
    history_size: int


@router.post("/abd/update", response_model=AbdUpdateResponse)
async def abd_update(req: AbdUpdateRequest) -> AbdUpdateResponse:
    state = AbdState(
        multiplier=req.current_multiplier,
        floor=req.floor,
        ceiling=req.ceiling,
    )
    state = update(
        state,
        acceptance_rate=req.acceptance_rate,
        error_rate=req.error_rate,
        restriction_count_24h=req.restriction_count_24h,
        base_daily_cap=req.base_daily_cap,
    )
    return AbdUpdateResponse(
        new_multiplier=state.multiplier,
        effective_daily_cap=effective_daily_cap(state, req.base_daily_cap),
        history_size=len(state.history),
    )
