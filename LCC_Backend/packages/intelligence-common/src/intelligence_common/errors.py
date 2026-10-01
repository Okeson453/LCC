"""Common error types for the Intelligence Engine."""

from __future__ import annotations


class IntelligenceEngineError(Exception):
    """Base exception for the Intelligence Engine."""


class LlmCallFailed(IntelligenceEngineError):
    """An LLM call failed (provider down, timeout, budget exceeded, etc.)."""


class RAGRetrievalError(IntelligenceEngineError):
    """RAG retrieval failed."""


class VectorStoreError(IntelligenceEngineError):
    """Vector store operation failed."""


class EmbeddingModelUnavailable(IntelligenceEngineError):
    """The embedding model couldn't be loaded."""
