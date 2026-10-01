"""re-embed-worker entrypoint.

Runs as a periodic background job. Scans the `kb_record_chunks` table for
records whose `embedding_model_version` differs from the current model
and re-embeds them in the vector store.
"""

from __future__ import annotations

import asyncio
import os
import time

from event_bus import Topic
from intelligence_common.config import Settings as BaseSettings
from intelligence_common.logging import configure_logging, get_logger
from pydantic_settings import SettingsConfigDict
from vector_db import QdrantClientWrapper

log = get_logger(__name__)


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="RE_EMBED_", extra="ignore")
    service_name: str = "re-embed-worker"
    log_level: str = "info"
    interval_seconds: int = 3600
    batch_size: int = 100
    current_model_version: str = "all-MiniLM-L6-v2@1"
    vector_db_url: str = "http://qdrant:6333"


async def _process_batch(settings: Settings) -> int:
    """Process one batch of stale embeddings. Returns the count processed."""
    # The "stale" detection would query the `kb_record_chunks` table for
    # rows where `embedding_model_version` != current_model_version.
    # In this stub we return 0 (no work to do).
    db = QdrantClientWrapper(url=settings.vector_db_url)
    # In production: select 100 chunk rows → re-embed → upsert → mark updated.
    return 0


async def main() -> None:
    settings = Settings()
    configure_logging(settings.service_name, settings.log_level)
    log.info("re_embed_worker.startup", interval=settings.interval_seconds)
    while True:
        try:
            processed = await _process_batch(settings)
            log.info("re_embed_worker.batch_complete", processed=processed)
        except Exception as e:
            log.error("re_embed_worker.batch_failed", error=str(e))
        await asyncio.sleep(settings.interval_seconds)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("re_embed_worker.shutdown")
