"""llm-client — async LLM provider clients (OpenAI, Anthropic).

Each provider implements the `LLMClient` protocol defined in `base.py`.
The cost controller (`cost.py`) selects a model tier based on monthly usage.
Retry policy (`retry.py`) handles transient errors with exponential backoff.
"""

from llm_client.base import (
    LLMClient,
    LLMMessage,
    LLMRequest,
    LLMResponse,
    ModelPricing,
)
from llm_client.cost import CostController, ModelTier
from llm_client.retry import RetryError, with_retry
from llm_client.openai_client import OpenAIClient
from llm_client.anthropic_client import AnthropicClient

__all__ = [
    "LLMClient",
    "LLMMessage",
    "LLMRequest",
    "LLMResponse",
    "ModelPricing",
    "CostController",
    "ModelTier",
    "RetryError",
    "with_retry",
    "OpenAIClient",
    "AnthropicClient",
]
