"""LLM client base types + abstract interface."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Optional


@dataclass
class LLMMessage:
    """A single chat message."""

    role: str  # "system" | "user" | "assistant"
    content: str


@dataclass
class LLMRequest:
    """A chat-completion request."""

    model_id: str
    messages: list[LLMMessage]
    temperature: float = 0.7
    max_tokens: Optional[int] = None
    top_p: Optional[float] = None
    stop: Optional[list[str]] = None
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class LLMResponse:
    """A chat-completion response."""

    completion: str
    model_id: str
    prompt_tokens: int
    completion_tokens: int
    cost_usd: float
    latency_ms: int
    raw: dict[str, Any] = field(default_factory=dict)


@dataclass
class ModelPricing:
    """Per-1K-token pricing for a model."""

    input_per_1k: float
    output_per_1k: float


class LLMClient(ABC):
    """Abstract LLM provider interface."""

    @abstractmethod
    async def complete(self, request: LLMRequest) -> LLMResponse:
        """Send a request, return the response."""

    async def close(self) -> None:
        """Release any resources."""
        return None
