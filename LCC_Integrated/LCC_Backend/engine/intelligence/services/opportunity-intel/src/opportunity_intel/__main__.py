"""opportunity-intel entrypoint."""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI

from intelligence_common.logging import configure_logging, get_logger
from opportunity_intel.api.discovery import router as discovery_router
from opportunity_intel.api.scoring import router as scoring_router
from opportunity_intel.config import Settings
from opportunity_intel.health import router as health_router

settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(title="opportunity-intel", version="0.1.0")
app.include_router(health_router)
app.include_router(discovery_router, prefix="/internal/intelligence")
app.include_router(scoring_router, prefix="/internal/intelligence")


@app.on_event("startup")
async def startup_event() -> None:
    log.info("opportunity_intel.startup", http_port=settings.http_port)


if __name__ == "__main__":
    uvicorn.run(
        "opportunity_intel.__main__:app",
        host="0.0.0.0",
        port=settings.http_port,
        reload=os.environ.get("LCC_DEV_RELOAD") == "1",
    )
