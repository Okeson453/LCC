"""enrichment-worker — consumes signal events and fetches third-party data.

Subscribes to:
- contact.created → fetch profile snapshot
- opportunity.discovered → fetch job/company data

Publishes:
- profile.snapshot.created
- kb.record.created (when new facts surface)

Enforces TTL: third-party data is stamped with TTL (per spec §6).
"""

from __future__ import annotations

import asyncio
import os
from datetime import UTC, datetime, timedelta

import httpx
from event_bus import (
    Consumer,
    Topic,
    EventEnvelope,
    Publisher,
)
from intelligence_common.logging import configure_logging, get_logger
from pydantic_settings import BaseSettings, SettingsConfigDict

log = get_logger(__name__)

# Third-party data TTL (per spec §6)
THIRD_PARTY_TTL_DAYS = 30


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="ENRICHMENT_", extra="ignore")
    service_name: str = "enrichment-worker"
    log_level: str = "info"
    redis_url: str = "redis://redis:6379"
    consumer_group: str = "enrichment-worker"
    consumer_name: str = "enrichment-1"
    third_party_api_url: str = ""
    third_party_api_key: str = ""


async def handle_contact_created(envelope: EventEnvelope, publisher: Publisher, settings: Settings) -> None:
    """Fetch profile snapshot for the new contact."""
    payload = envelope.payload
    contact_id = payload.get("contact_id")
    if not contact_id:
        return
    if not settings.third_party_api_url:
        return

    now = datetime.now(UTC)
    expires_at = now + timedelta(days=THIRD_PARTY_TTL_DAYS)

    async with httpx.AsyncClient(timeout=10.0) as client:
        try:
            resp = await client.get(
                f"{settings.third_party_api_url}/contacts/{contact_id}",
                headers={"Authorization": f"Bearer {settings.third_party_api_key}"} if settings.third_party_api_key else {},
            )
            resp.raise_for_status()
            data = resp.json()
        except (httpx.HTTPError, httpx.TimeoutException) as e:
            log.warning("enrichment_worker.contact_fetch_failed", contact_id=contact_id, error=str(e))
            return

    # Publish profile.snapshot.created with TTL.
    await publisher.publish_typed(
        topic=Topic.PROFILE_SNAPSHOT_CREATED,
        producer=settings.service_name,
        trace_id=envelope.header.trace_id,
        payload={
            "contact_id": contact_id,
            "snapshot_kind": "third_party",
            "data": data,
            "third_party_ttl_expires_at": expires_at.isoformat(),
            "fetched_at": now.isoformat(),
        },
        member_id=envelope.header.member_id,
    )


async def handle_opportunity_discovered(envelope: EventEnvelope, publisher: Publisher, settings: Settings) -> None:
    payload = envelope.payload
    contact_id = payload.get("contact_id")
    if not contact_id or not settings.third_party_api_url:
        return

    now = datetime.now(UTC)
    expires_at = now + timedelta(days=THIRD_PARTY_TTL_DAYS)

    async with httpx.AsyncClient(timeout=10.0) as client:
        try:
            resp = await client.get(
                f"{settings.third_party_api_url}/contacts/{contact_id}/context",
                headers={"Authorization": f"Bearer {settings.third_party_api_key}"} if settings.third_party_api_key else {},
            )
            resp.raise_for_status()
            data = resp.json()
        except (httpx.HTTPError, httpx.TimeoutException) as e:
            log.warning("enrichment_worker.opportunity_fetch_failed", contact_id=contact_id, error=str(e))
            return

    await publisher.publish_typed(
        topic=Topic.PROFILE_SNAPSHOT_CREATED,
        producer=settings.service_name,
        trace_id=envelope.header.trace_id,
        payload={
            "contact_id": contact_id,
            "snapshot_kind": "opportunity_context",
            "data": data,
            "third_party_ttl_expires_at": expires_at.isoformat(),
            "fetched_at": now.isoformat(),
        },
        member_id=envelope.header.member_id,
    )


async def main() -> None:
    settings = Settings()
    configure_logging(settings.service_name, settings.log_level)
    log.info("enrichment_worker.startup")

    async with Publisher(settings.redis_url) as publisher, Consumer(
        redis_url=settings.redis_url,
        group=settings.consumer_group,
        consumer_name=settings.consumer_name,
    ) as consumer:
        # Handlers bound to closures.
        async def on_contact(env: EventEnvelope) -> None:
            await handle_contact_created(env, publisher, settings)

        async def on_opportunity(env: EventEnvelope) -> None:
            await handle_opportunity_discovered(env, publisher, settings)

        consumer.on(Topic.MEMBER_CREATED, on_contact)
        consumer.on(Topic.OPPORTUNITY_DISCOVERED, on_opportunity)
        await consumer.run(poll_interval_seconds=2.0)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        log.info("enrichment_worker.shutdown")
