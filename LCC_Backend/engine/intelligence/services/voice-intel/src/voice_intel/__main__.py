"""voice-intel entrypoint."""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI

from intelligence_common.logging import configure_logging, get_logger
from voice_intel.api.fingerprint import router as fingerprint_router
from voice_intel.api.samples import router as samples_router
from voice_intel.config import Settings
from voice_intel.health import router as health_router

settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(title="voice-intel", version="0.1.0")
app.include_router(health_router)
app.include_router(samples_router, prefix="/internal/intelligence/voice")
app.include_router(fingerprint_router, prefix="/internal/intelligence/voice")


@app.on_event("startup")
async def startup_event() -> None:
    log.info("voice_intel.startup", http_port=settings.http_port)


if __name__ == "__main__":
    uvicorn.run(
        "voice_intel.__main__:app",
        host="0.0.0.0",
        port=settings.http_port,
        reload=os.environ.get("LCC_DEV_RELOAD") == "1",
    )
