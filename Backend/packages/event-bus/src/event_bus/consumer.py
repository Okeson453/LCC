"""Consumer — subscribe to a Redis Stream + dispatch to a handler.

At-least-once delivery with idempotency dedup via Redis SET NX with 7-day TTL.
"""

from __future__ import annotations

import asyncio
import json
from typing import Any, Awaitable, Callable

import redis.asyncio as redis_asyncio
import structlog

from event_bus.envelope import EventEnvelope, Topic

log = structlog.get_logger(__name__)


class ConsumerError(Exception):
    pass


Handler = Callable[[EventEnvelope], Awaitable[None]]


class Consumer:
    """Async Redis Streams consumer with idempotent dispatch."""

    def __init__(
        self,
        redis_url: str,
        group: str,
        consumer_name: str,
        stream_prefix: str = "lcc",
    ) -> None:
        self._redis_url = redis_url
        self._group = group
        self._consumer_name = consumer_name
        self._stream_prefix = stream_prefix
        self._client: redis_asyncio.Redis | None = None
        self._handlers: dict[Topic, Handler] = {}
        self._stopping = asyncio.Event()

    def on(self, topic: Topic, handler: Handler) -> None:
        self._handlers[topic] = handler

    async def connect(self) -> None:
        self._client = redis_asyncio.from_url(self._redis_url, decode_responses=True)

    async def close(self) -> None:
        if self._client:
            await self._client.aclose()

    async def stop(self) -> None:
        self._stopping.set()

    async def __aenter__(self) -> Consumer:
        await self.connect()
        return self

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        await self.close()

    async def ensure_group(self, topic: Topic) -> None:
        """Ensure the consumer group exists (XGROUP CREATE … MKSTREAM)."""
        assert self._client is not None
        stream_key = f"{self._stream_prefix}:{topic.value}"
        try:
            await self._client.xgroup_create(stream_key, self._group, id="$", mkstream=True)
        except redis_asyncio.ResponseError as e:
            if "BUSYGROUP" not in str(e):
                raise ConsumerError(f"xgroup_create failed: {e}") from e

    async def poll_once(self, batch_size: int = 16) -> int:
        """One poll cycle. Returns number of events processed."""
        assert self._client is not None
        processed = 0
        for topic, handler in self._handlers.items():
            stream_key = f"{self._stream_prefix}:{topic.value}"
            try:
                entries = await self._client.xreadgroup(
                    groupname=self._group,
                    consumername=self._consumer_name,
                    streams={stream_key: ">"},
                    count=batch_size,
                    block=0,
                )
            except redis_asyncio.RedisError as e:
                log.warning("xreadgroup_failed", topic=topic.value, error=str(e))
                continue

            for _stream, msgs in entries:
                for msg_id, fields in msgs:
                    if await self._dispatch(msg_id, fields, topic, handler):
                        processed += 1
                        try:
                            await self._client.xack(stream_key, self._group, msg_id)
                        except redis_asyncio.RedisError as e:
                            log.warning("xack_failed", msg_id=msg_id, error=str(e))
        return processed

    async def _dispatch(
        self,
        msg_id: str,
        fields: dict[str, Any],
        topic: Topic,
        handler: Handler,
    ) -> bool:
        # Idempotency dedup.
        idem_key = fields.get("idempotency_key")
        if idem_key:
            dedup_key = f"dedup:{self._group}:{idem_key}"
            set_result = await self._client.set(dedup_key, "1", ex=7 * 24 * 3600, nx=True)
            if not set_result:
                log.debug("event_already_processed", msg_id=msg_id, idem_key=idem_key)
                return True

        # Reconstruct the envelope.
        try:
            data = json.loads(fields.get("data", "{}"))
        except json.JSONDecodeError as e:
            log.warning("event_parse_failed", msg_id=msg_id, error=str(e))
            return False

        envelope = EventEnvelope(
            header=type("Hdr", (), {
                "event_id": fields.get("event_id", msg_id),
                "topic": topic,
                "trace_id": fields.get("trace_id", ""),
                "producer_service": fields.get("producer", ""),
                "occurred_at": fields.get("occurred_at", ""),
                "member_id": fields.get("member_id") or None,
                "idempotency_key": idem_key or msg_id,
                "schema_version": int(fields.get("schema_version", "1")),
            })(),
            payload=data,
        )

        try:
            await handler(envelope)
            return True
        except Exception as e:  # noqa: BLE001
            log.error(
                "event_handler_failed",
                msg_id=msg_id,
                topic=topic.value,
                error=str(e),
            )
            return False

    async def run(self, poll_interval_seconds: float = 1.0) -> None:
        """Run the consumer loop until stopped."""
        for topic in self._handlers:
            await self.ensure_group(topic)

        while not self._stopping.is_set():
            try:
                await self.poll_once()
            except Exception as e:  # noqa: BLE001
                log.error("consumer_poll_failed", error=str(e))
            try:
                await asyncio.wait_for(self._stopping.wait(), timeout=poll_interval_seconds)
            except asyncio.TimeoutError:
                pass
