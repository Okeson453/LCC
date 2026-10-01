"""LLM client wrapper — picks OpenAI vs Anthropic based on the model tier."""

from __future__ import annotations

from llm_client import (
    AnthropicClient,
    LLMClient,
    LLMMessage,
    LLMRequest,
    LLMResponse,
    OpenAIClient,
)


# Model tier → provider + model_id
TIER_MODELS: dict[str, tuple[str, str]] = {
    "premium": ("openai", "gpt-4o-2024-08-06"),
    "standard": ("anthropic", "claude-sonnet-4-20250514"),
    "cheap": ("anthropic", "claude-haiku-4-20250514"),
    "rule_based": ("none", ""),
}


_clients: dict[str, LLMClient] = {}


def _get_client(provider: str) -> LLMClient:
    """Get-or-create a client for the given provider."""
    if provider in _clients:
        return _clients[provider]
    if provider == "openai":
        import os
        client: LLMClient = OpenAIClient(api_key=os.environ.get("OPENAI_API_KEY", ""))
    elif provider == "anthropic":
        import os
        client = AnthropicClient(api_key=os.environ.get("ANTHROPIC_API_KEY", ""))
    else:
        raise ValueError(f"unsupported provider: {provider}")
    _clients[provider] = client
    return client


async def complete(prompt: str, model_tier: str) -> LLMResponse:
    """Send a prompt to the appropriate model for the tier."""
    if model_tier == "rule_based":
        raise ValueError("rule_based tier does not support complete() — use rule-based fallbacks")
    provider, model_id = TIER_MODELS[model_tier]
    client = _get_client(provider)
    request = LLMRequest(
        model_id=model_id,
        messages=[LLMMessage(role="user", content=prompt)],
        temperature=0.7,
        max_tokens=2048,
    )
    return await client.complete(request)
