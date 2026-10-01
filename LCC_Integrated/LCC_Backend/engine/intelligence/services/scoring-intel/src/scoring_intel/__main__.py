"""scoring-intel entrypoint."""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI

from intelligence_common.logging import configure_logging, get_logger
from scoring_intel.api.hc import router as hc_router
from scoring_intel.api.abd import router as abd_router
from scoring_intel.api.rho import router as rho_router
from scoring_intel.config import Settings
from scoring_intel.health import router as health_router

settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(title="scoring-intel", version="0.1.0")
app.include_router(health_router)
app.include_router(rho_router, prefix="/internal/intelligence/scoring")
app.include_router(abd_router, prefix="/internal/intelligence/scoring")
app.include_router(hc_router, prefix="/internal/intelligence/scoring")


@app.on_event("startup")
async def startup_event() -> None:
    log.info("scoring_intel.startup", http_port=settings.http_port)


if __name__ == "__main__":
    uvicorn.run(
        "scoring_intel.__main__:app",
        host="0.0.0.0",
        port=settings.http_port,
        reload=os.environ.get("LCC_DEV_RELOAD") == "1",
    )
