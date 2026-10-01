"""Shared base config for Intelligence Engine services."""

from __future__ import annotations

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Base settings — every service should subclass and override env_prefix."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    service_name: str = "intelligence-service"
    service_version: str = "0.1.0"
    environment: str = "local"
    log_level: str = "info"

    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"
    redis_url: str = "redis://redis:6379"
    vector_db_url: str = "http://qdrant:6333"

    otlp_endpoint: str | None = None
    audit_svc_url: str = "http://audit-svc:8080"

    # Service-specific knobs default to safe values.
    monthly_budget_usd: float = 500.0
