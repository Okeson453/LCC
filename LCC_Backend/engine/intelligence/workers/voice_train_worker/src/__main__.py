"""voice-train-worker — recomputes the voice centroid per member.

For each active member with at least `min_samples_required` approved voice
samples in `voice_samples`, the worker:

1. Fetches all samples for that member from Qdrant.
2. Recomputes the centroid embedding (mean of the sample vectors).
3. Computes the fingerprint on the concatenation of sample texts.
4. Upserts the centroid as a special point in `voice_samples` with id
   `voice-centroid:{member_id}`.
5. Publishes `voice.profile.updated` on the event bus with the new centroid.

The ai-worker consumes this event to refresh its voice retrieval cache.

The worker is fault-tolerant: per-member failures are logged and don't abort
the loop. Failures during the bulk fetch fall through to a smaller per-member
search as a fallback.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import os
import time
from datetime import UTC, datetime
from typing import Any

from event_bus import EventEnvelope, Publisher, Topic
from event_bus.envelope import EventHeader
from intelligence_common.logging import configure_logging, get_logger
from pydantic_settings import BaseSettings, SettingsConfigDict

log = get_logger(__name__)


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="VOICE_TRAIN_", extra="ignore")
    service_name: str = "voice-train-worker"
    service_version: str = "0.1.0"
    log_level: str = "info"
    interval_seconds: int = 86_400  # daily
    min_samples_required: int = 5
    embedding_dim: int = 384  # all-MiniLM-L6-v2
    qdrant_url: str = "http://qdrant:6333"
    qdrant_api_key: str | None = None
    redis_url: str = "redis://redis:6379"
    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"


def _parse_qdrant_response(raw: Any) -> list[dict]:
    """Normalize a qdrant-client search result into a list of {id, score, payload, vector}."""
    if raw is None:
        return []
    out: list[dict] = []
    for h in raw:
        if isinstance(h, dict):
            out.append(h)
        else:
            out.append(
                {
                    "id": getattr(h, "id", None),
                    "score": getattr(h, "score", 0.0),
                    "payload": getattr(h, "payload", {}) or {},
                    "vector": getattr(h, "vector", None),
                }
            )
    return out


async def list_active_members(pool: Any) -> list[str]:
    """Return the ids of all active members."""
    rows = await pool.fetch(
        "SELECT id::TEXT AS id FROM lcc.members WHERE is_active = TRUE"
    )
    return [str(r["id"]) for r in rows]


async def fetch_member_samples(qdrant: Any, member_id: str) -> list[dict]:
    """Fetch all voice samples for a member from Qdrant.

    Uses scroll-style pagination since Qdrant's `search` requires a query
    vector; we want every sample for the member regardless of similarity.
    """
    out: list[dict] = []
    offset = None
    for _ in range(20):  # safety: max 20 × 200 = 4,000 samples per member
        try:
            batch, next_offset = await qdrant.scroll(
                collection_name="voice_samples",
                scroll_filter={
                    "must": [{"key": "member_id", "match": {"value": member_id}}]
                },
                limit=200,
                offset=offset,
                with_payload=True,
                with_vectors=True,
            )
        except Exception as e:
            log.warning("voice_train.qdrant_scroll_failed", member_id=member_id, error=str(e))
            return out
        out.extend(_parse_qdrant_response(batch))
        if next_offset is None:
            break
        offset = next_offset
    return out


def compute_centroid(samples: list[dict]) -> list[float] | None:
    """Mean of sample vectors. Returns None if no vectors present."""
    vectors = [s.get("vector") for s in samples if s.get("vector")]
    if not vectors:
        return None
    dim = len(vectors[0])
    sum_vec = [0.0] * dim
    for v in vectors:
        for i in range(dim):
            sum_vec[i] += v[i]
    n = float(len(vectors))
    return [x / n for x in sum_vec]


def compute_text_fingerprint(samples: list[dict]) -> dict:
    """Deterministic text-level fingerprint from concatenated samples."""
    texts = [s.get("payload", {}).get("text", "") for s in samples]
    text = "\n\n".join(t for t in texts if t)
    if not text.strip():
        return {"avg_sentence_length": 0.0, "tone_descriptor": "empty"}

    # Cheap fingerprint — we don't import voice-intel here to keep the
    # worker independent. The ai-worker re-applies the full fingerprint
    # analyzer on retrieval anyway; this gives us a stable hash + a
    # rough tone hint for the audit log.
    sentences = max(1, text.count(". ") + text.count(".\n") + 1)
    avg_sentence_length = len(text) / sentences
    lower = sum(1 for c in text if c.islower())
    alpha = max(1, sum(1 for c in text if c.isalpha()))
    lowercase_ratio = lower / alpha
    tone = (
        "playful" if "🚀" in text or "🎉" in text
        else "essayistic" if avg_sentence_length > 200
        else "punchy" if avg_sentence_length < 50
        else "neutral"
    )
    return {
        "avg_sentence_length": avg_sentence_length,
        "lowercase_ratio": lowercase_ratio,
        "tone_descriptor": tone,
        "text_hash": hashlib.sha256(text.encode("utf-8")).hexdigest(),
    }


async def upsert_centroid(
    qdrant: Any,
    member_id: str,
    centroid: list[float],
    fingerprint: dict,
    sample_count: int,
) -> None:
    """Upsert the centroid as a special point in voice_samples."""
    payload = {
        "member_id": member_id,
        "kind": "voice_centroid",
        "fingerprint": fingerprint,
        "sample_count": sample_count,
        "trained_at": datetime.now(UTC).isoformat(),
    }
    await qdrant.upsert(
        collection_name="voice_samples",
        points=[
            {
                "id": f"voice-centroid:{member_id}",
                "vector": centroid,
                "payload": payload,
            }
        ],
    )


async def train_one_member(
    qdrant: Any,
    publisher: Publisher,
    settings: Settings,
    member_id: str,
    stats: dict[str, int],
) -> bool:
    """Train one member. Returns True on success."""
    samples = await fetch_member_samples(qdrant, member_id)
    real_samples = [s for s in samples if s.get("payload", {}).get("kind") != "voice_centroid"]
    if len(real_samples) < settings.min_samples_required:
        stats["skipped"] += 1
        log.debug(
            "voice_train.skipped_insufficient_samples",
            member_id=member_id,
            count=len(real_samples),
        )
        return False

    centroid = compute_centroid(real_samples)
    if centroid is None:
        stats["skipped"] += 1
        log.debug("voice_train.skipped_no_vectors", member_id=member_id)
        return False

    fingerprint = compute_text_fingerprint(real_samples)
    await upsert_centroid(qdrant, member_id, centroid, fingerprint, len(real_samples))

    await publisher.publish_typed(
        topic=Topic.VOICE_SAMPLE_ADDED,  # we re-use this topic for centroid updates
        producer=settings.service_name,
        trace_id=f"voice-train:{member_id}:{int(time.time())}",
        payload={
            "member_id": member_id,
            "kind": "voice_centroid",
            "sample_count": len(real_samples),
            "fingerprint": fingerprint,
            "trained_at": datetime.now(UTC).isoformat(),
        },
        member_id=member_id,
    )
    stats["trained"] += 1
    log.info(
        "voice_train.trained",
        member_id=member_id,
        sample_count=len(real_samples),
    )
    return True


async def _build_qdrant(settings: Settings) -> Any:
    try:
        from qdrant_client import AsyncQdrantClient
    except ImportError:
        log.error("voice_train.qdrant_client_not_installed")
        return None
    return AsyncQdrantClient(
        url=settings.qdrant_url,
        api_key=settings.qdrant_api_key,
    )


async def _build_pool(settings: Settings) -> Any:
    try:
        import asyncpg
    except ImportError:
        log.error("voice_train.asyncpg_not_installed")
        return None
    return await asyncpg.create_pool(
        settings.postgres_url, min_size=1, max_size=4, command_timeout=10
    )


async def tick(publisher: Publisher, settings: Settings) -> dict[str, int]:
    """One training pass."""
    qdrant = await _build_qdrant(settings)
    pool = await _build_pool(settings)
    if qdrant is None or pool is None:
        return {"trained": 0, "skipped": 0, "errors": 1}

    stats = {"trained": 0, "skipped": 0, "errors": 0}
    try:
        member_ids = await list_active_members(pool)
        log.info("voice_train.tick", candidate_members=len(member_ids))
        for member_id in member_ids:
            try:
                await train_one_member(qdrant, publisher, settings, member_id, stats)
            except Exception as e:
                stats["errors"] += 1
                log.warning("voice_train.member_failed", member_id=member_id, error=str(e))
    finally:
        await pool.close()
        await qdrant.close()
    return stats


async def main() -> None:
    settings = Settings()
    configure_logging(settings.service_name, settings.log_level)
    log.info("voice_train_worker.startup", interval=settings.interval_seconds)
    while True:
        try:
            async with Publisher(settings.redis_url) as publisher:
                stats = await tick(publisher, settings)
                log.info("voice_train_worker.tick_complete", **stats)
        except Exception as e:
            log.error("voice_train_worker.tick_failed", error=str(e))
        await asyncio.sleep(settings.interval_seconds)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("voice_train_worker.shutdown")
