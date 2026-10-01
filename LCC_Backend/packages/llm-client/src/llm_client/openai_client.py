"""OpenAI provider implementation."""

from __future__ import annotations

import os
import time
from typing import Any

import httpx

from llm_client.base import LLMClient, LLMRequest, LLMResponse, ModelPricing


class OpenAIClient(LLMClient):
    """Async client for the OpenAI chat-completions API.

    Pricing is per 1K tokens; cost_usd = tokens * price / 1000.
    """

    BASE_URL = "https://api.openai.com/v1/chat/completions"

    PRICING: dict[str, ModelPricing] = {
        "gpt-4o-2024-08-06": ModelPricing(input_per_1k=0.005, output_per_1k=0.015),
        "gpt-4o-mini-2024-07-18": ModelPricing(input_per_1k=0.00015, output_per_1k=0.0006),
        "gpt-4-turbo-2024-04-09": ModelPricing(input_per_1k=0.01, output_per_1k=0.03),
        "o1-preview-2024-09-12": ModelPricing(input_per_1k=0.015, output_per_1k=0.06),
    }

    def __init__(self, api_key: str | None = None, timeout_seconds: float = 60.0) -> None:
        self._api_key = api_key or os.environ.get("OPENAI_API_KEY", "")
        self._http = httpx.AsyncClient(timeout=timeout_seconds)

    async def complete(self, request: LLMRequest) -> LLMResponse:
        if not self._api_key:
            raise RuntimeError("OPENAI_API_KEY not configured")
        headers = {
            "Authorization": f"Bearer {self._api_key}",
            "Content-Type": "application/json",
        }
        body: dict[str, Any] = {
            "model": request.model_id,
            "messages": [{"role": m.role, "content": m.content} for m in request.messages],
            "temperature": request.temperature,
        }
        if request.max_tokens:
            body["max_tokens"] = request.max_tokens
        if request.top_p:
            body["top_p"] = request.top_p
        if request.stop:
            body["stop"] = request.stop

        start = time.monotonic()
        try:
            resp = await self._http.post(self.BASE_URL, headers=headers, json=body)
        except httpx.HTTPError as e:
            raise RuntimeError(f"OpenAI HTTP error: {e}") from e
        latency_ms = int((time.monotonic() - start) * 1000)

        if resp.status_code != 200:
            raise RuntimeError(f"OpenAI API returned {resp.status_code}: {resp.text}")

        data = resp.json()
        completion = data["choices"][0]["message"]["content"]
        usage = data.get("usage", {})
        prompt_tokens = int(usage.get("prompt_tokens", 0))
        completion_tokens = int(usage.get("completion_tokens", 0))
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
            # Unknown model: estimate at GPT-4o pricing.
            pricing = self.PRICING["gpt-4o-2024-08-06"]
        return (prompt_tokens * pricing.input_per_1k + completion_tokens * pricing.output_per_1k) / 1000.0

    async def close(self) -> None:
        await self._http.aclose()
