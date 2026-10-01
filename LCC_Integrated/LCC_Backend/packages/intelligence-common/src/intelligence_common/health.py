"""Shared health-check helpers."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Awaitable, Callable

import httpx

from intelligence_common.logging import get_logger

log = get_logger(__name__)


@dataclass
class HealthStatus:
    """Result of a health check."""

    service: str
    healthy: bool
    checks: dict[str, bool]
    note: str | None = None


class HealthCheck:
    """Composable health check runner."""

    def __init__(self, service: str) -> None:
        self.service = service
        self._checks: dict[str, Callable[[], Awaitable[bool]]] = {}

    def register(self, name: str, check: Callable[[], Awaitable[bool]]) -> None:
        self._checks[name] = check

    async def run(self) -> HealthStatus:
        results: dict[str, bool] = {}
        all_healthy = True
        for name, check in self._checks.items():
            try:
                results[name] = bool(await check())
            except Exception as e:  # noqa: BLE001
                log.warning("health_check_failed", name=name, error=str(e))
                results[name] = False
            if not results[name]:
                all_healthy = False
        return HealthStatus(service=self.service, healthy=all_healthy, checks=results)


async def ready_check(url: str, timeout_seconds: float = 2.0) -> bool:
    """Poll a /healthz or /readyz endpoint."""
    try:
        async with httpx.AsyncClient(timeout=timeout_seconds) as client:
            r = await client.get(url)
            return r.status_code == 200
    except (httpx.HTTPError, httpx.TimeoutException):
        return False
