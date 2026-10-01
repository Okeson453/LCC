"""Anthropic provider implementation."""

from __future__ import annotations

import os
import time
from typing import Any

import httpx

from llm_client.base import LLMClient, LLMRequest, LLMResponse, ModelPricing


class AnthropicClient(LLMClient):
    """Async client for the Anthropic messages API.

    Pricing is per 1K tokens; cost_usd = tokens * price / 1000.
    """

    BASE_URL = "https://api.anthropic.com/v1/messages"
    API_VERSION = "2023-06-01"

    PRICING: dict[str, ModelPricing] = {
        "claude-sonnet-4-20250514": ModelPricing(input_per_1k=0.003, output_per_1k=0.015),
        "claude-haiku-4-20250514": ModelPricing(input_per_1k=0.0008, output_per_1k=0.004),
        "claude-opus-4-20250912": ModelPricing(input_per_1k=0.015, output_per_1k=0.075),
    }

    def __init__(self, api_key: str | None = None, timeout_seconds: float = 60.0) -> None:
        self._api_key = api_key or os.environ.get("ANTHROPIC_API_KEY", "")
        self._http = httpx.AsyncClient(timeout=timeout_seconds)

    async def complete(self, request: LLMRequest) -> LLMResponse:
        if not self._api_key:
            raise RuntimeError("ANTHROPIC_API_KEY not configured")
        headers = {
            "x-api-key": self._api_key,
            "anthropic-version": self.API_VERSION,
            "Content-Type": "application/json",
        }

        # Anthropic separates system from user messages.
        system_text = ""
        user_messages: list[dict[str, str]] = []
        for m in request.messages:
            if m.role == "system":
                system_text += m.content + "\n"
            else:
                user_messages.append({"role": m.role, "content": m.content})

        body: dict[str, Any] = {
            "model": request.model_id,
            "max_tokens": request.max_tokens or 2048,
            "messages": user_messages,
        }
        if system_text:
            body["system"] = system_text.strip()
        if request.temperature is not None:
            body["temperature"] = request.temperature
        if request.top_p is not None:
            body["top_p"] = request.top_p
        if request.stop:
            body["stop_sequences"] = request.stop

        start = time.monotonic()
        try:
            resp = await self._http.post(self.BASE_URL, headers=headers, json=body)
        except httpx.HTTPError as e:
            raise RuntimeError(f"Anthropic HTTP error: {e}") from e
        latency_ms = int((time.monotonic() - start) * 1000)

        if resp.status_code != 200:
            raise RuntimeError(f"Anthropic API returned {resp.status_code}: {resp.text}")

        data = resp.json()
        # Anthropic returns a list of content blocks.
        content_blocks = data.get("content", [])
        completion = "".join(b.get("text", "") for b in content_blocks if b.get("type") == "text")
        usage = data.get("usage", {})
        prompt_tokens = int(usage.get("input_tokens", 0))
        completion_tokens = int(usage.get("output_tokens", 0))
        cost_usd = self._compute_cost(request.model_id, prompt_tokens, completion_tokens)

        return LLMResponse(
            completion=completion,
            model_id=request.model_id,
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
            cost_usd=cost_usd,
            latency_ms=latency_ms,
            raw=data,
        )

    def _compute_cost(self, model_id: str, prompt_tokens: int, completion_tokens: int) -> float:
        pricing = self.PRICING.get(model_id)
        if pricing is None:
            pricing = self.PRICING["claude-sonnet-4-20250514"]
        return (prompt_tokens * pricing.input_per_1k + completion_tokens * pricing.output_per_1k) / 1000.0

    async def close(self) -> None:
        await self._http.aclose()
