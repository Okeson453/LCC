"""ai-worker entrypoint."""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI

from ai_worker.api.drafts import router as drafts_router
from ai_worker.api.embedding import router as embedding_router
from ai_worker.config import Settings
from ai_worker.health import router as health_router
from intelligence_common.logging import configure_logging, get_logger

settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(
    title="ai-worker",
    version="0.1.0",
    description="OKESON-LCC AI/RAG Worker — LLM orchestration, RAG, brand guard",
)

# Routers.
app.include_router(health_router)
app.include_router(drafts_router, prefix="/internal/llm")
app.include_router(embedding_router, prefix="/internal/llm")


@app.on_event("startup")
async def startup_event() -> None:
    log.info("ai_worker.startup", http_port=settings.http_port)


if __name__ == "__main__":
    uvicorn.run(
        "ai_worker.__main__:app",
        host="0.0.0.0",
        port=settings.http_port,
        reload=os.environ.get("LCC_DEV_RELOAD") == "1",
    )
