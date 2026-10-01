"""Publisher — XADD to Redis Streams."""

from __future__ import annotations

import json
from typing import Any

import redis.asyncio as redis_asyncio
import structlog

from event_bus.envelope import EventEnvelope, Topic

log = structlog.get_logger(__name__)


class PublisherError(Exception):
    """Publisher failed to deliver an event."""


class Publisher:
    """Async publisher to Redis Streams (Phase 1-2)."""

    def __init__(
        self,
        redis_url: str,
        stream_prefix: str = "lcc",
    ) -> None:
        self._redis_url = redis_url
        self._stream_prefix = stream_prefix
        self._client: redis_asyncio.Redis | None = None

    async def connect(self) -> None:
        self._client = redis_asyncio.from_url(self._redis_url, decode_responses=True)

    async def close(self) -> None:
        if self._client:
            await self._client.aclose()

    async def __aenter__(self) -> Publisher:
        await self.connect()
        return self

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        await self.close()

    async def publish(self, envelope: EventEnvelope) -> str:
        """Publish an envelope. Returns the Redis stream message ID."""
        if self._client is None:
            raise RuntimeError("Publisher must be used as async context manager")
        stream_key = f"{self._stream_prefix}:{envelope.header.topic.value}"
        fields = {
            "event_id": envelope.header.event_id,
            "topic": envelope.header.topic.value,
            "trace_id": envelope.header.trace_id,
            "producer": envelope.header.producer_service,
            "member_id": envelope.header.member_id or "",
            "schema_version": str(envelope.header.schema_version),
            "idempotency_key": envelope.header.idempotency_key,
            "data": json.dumps(envelope.payload),
        }
        try:
            stream_id = await self._client.xadd(stream_key, fields)
        except redis_asyncio.RedisError as e:
            raise PublisherError(f"xadd failed for {stream_key}: {e}") from e
        log.debug(
            "event_published",
            topic=envelope.header.topic.value,
            stream_id=stream_id,
            event_id=envelope.header.event_id,
        )
        return stream_id if isinstance(stream_id, str) else stream_id.decode()

    async def publish_typed(
        self,
        topic: Topic,
        producer: str,
        trace_id: str,
        payload: dict[str, Any],
        member_id: str | None = None,
    ) -> str:
        """Convenience: build + publish."""
        env = EventEnvelope.new(topic, producer, trace_id, payload, member_id)
        return await self.publish(env)
