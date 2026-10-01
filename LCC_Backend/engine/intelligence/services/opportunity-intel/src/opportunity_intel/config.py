"""opportunity-intel settings."""

from __future__ import annotations

from typing import Literal

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="OPPORTUNITY_INTEL_", extra="ignore")

    service_name: str = "opportunity-intel"
    service_version: str = "0.1.0"
    environment: Literal["local", "dev", "staging", "production"] = "local"
    log_level: str = "info"
    http_port: int = 8090

    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"
    redis_url: str = "redis://redis:6379"
    vector_db_url: str = "http://qdrant:6333"

    # Discovery configuration
    discovery_min_signal_score: float = 0.4
    discovery_max_signals_per_cycle: int = 100

    # φ scoring thresholds
    phi_qualified_threshold: float = 0.65
    phi_labeled_send_threshold: int = 200  # < this → rule-based fallback
