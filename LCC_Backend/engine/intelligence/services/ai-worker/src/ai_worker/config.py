"""ai-worker settings."""

from __future__ import annotations

from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="AI_WORKER_", extra="ignore")

    service_name: str = "ai-worker"
    service_version: str = "0.1.0"
    environment: Literal["local", "dev", "staging", "production"] = "local"
    log_level: str = "info"
    http_port: int = 8080
    grpc_port: int = 50051

    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"
    redis_url: str = "redis://redis:6379"
    vector_db_url: str = "http://qdrant:6333"

    openai_api_key: str = ""
    anthropic_api_key: str = ""

    # KB grounding defaults
    kb_top_k: int = 5
    voice_top_k: int = 3

    # Cost controller
    monthly_budget_usd: float = Field(default=500.0, gt=0)

    otlp_endpoint: str | None = None
    audit_svc_url: str = "http://audit-svc:8080"
