"""intelligence-common — shared utilities for the Python Intelligence Engine.

Modules:
- config: BaseSettings wrapper.
- logging: structlog configuration.
- telemetry: OpenTelemetry init.
- health: shared health-check helpers.
- errors: common exception types.
- audit_emitter: LLM-call audit emission.
"""

from intelligence_common.config import Settings
from intelligence_common.errors import (
    IntelligenceEngineError,
    LlmCallFailed,
    RAGRetrievalError,
    VectorStoreError,
)
from intelligence_common.health import HealthCheck, HealthStatus, ready_check
from intelligence_common.logging import configure_logging, get_logger

__all__ = [
    "Settings",
    "IntelligenceEngineError",
    "LlmCallFailed",
    "RAGRetrievalError",
    "VectorStoreError",
    "HealthCheck",
    "HealthStatus",
    "ready_check",
    "configure_logging",
    "get_logger",
]
