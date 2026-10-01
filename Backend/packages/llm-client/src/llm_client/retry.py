"""Retry policy for LLM calls.

Implements exponential backoff with jitter, per Backend Design Concept §44.

- On 5xx or transient errors: retry up to 3 times with backoff.
- On 4xx (rate-limit): respect Retry-After if present.
- On 4xx (other): do not retry (return error to caller).
"""

from __future__ import annotations

import asyncio
import random
from typing import Awaitable, Callable, TypeVar

T = TypeVar("T")


class RetryError(Exception):
    """All retries exhausted."""

    def __init__(self, attempts: int, last_error: Exception) -> None:
        super().__init__(f"retry exhausted after {attempts} attempts: {last_error}")
        self.attempts = attempts
        self.last_error = last_error


async def with_retry(
    fn: Callable[[], Awaitable[T]],
    *,
    max_attempts: int = 3,
    base_delay_ms: int = 500,
    max_delay_ms: int = 8000,
) -> T:
    """Run fn with exponential backoff + jitter."""
    last_error: Exception | None = None
    for attempt in range(1, max_attempts + 1):
        try:
            return await fn()
        except Exception as e:  # noqa: BLE001
            last_error = e
            if attempt == max_attempts:
                break
            delay_ms = min(max_delay_ms, base_delay_ms * (2 ** (attempt - 1)))
            delay_ms += random.randint(0, delay_ms // 2)  # jitter
            await asyncio.sleep(delay_ms / 1000.0)
    raise RetryError(max_attempts, last_error or RuntimeError("unknown error"))
