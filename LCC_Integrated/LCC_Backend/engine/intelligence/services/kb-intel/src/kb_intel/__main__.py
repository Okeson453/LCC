"""kb-intel entrypoint."""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI

from intelligence_common.logging import configure_logging, get_logger
from kb_intel.api.dedup import router as dedup_router
from kb_intel.api.ingest import router as ingest_router
from kb_intel.api.search import router as search_router
from kb_intel.config import Settings
from kb_intel.health import router as health_router

settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(title="kb-intel", version="0.1.0")
app.include_router(health_router)
app.include_router(ingest_router, prefix="/internal/intelligence/kb")
app.include_router(search_router, prefix="/internal/intelligence/kb")
app.include_router(dedup_router, prefix="/internal/intelligence/kb")


@app.on_event("startup")
async def startup_event() -> None:
    log.info("kb_intel.startup", http_port=settings.http_port)


if __name__ == "__main__":
    uvicorn.run(
        "kb_intel.__main__:app",
        host="0.0.0.0",
        port=settings.http_port,
        reload=os.environ.get("LCC_DEV_RELOAD") == "1",
    )
