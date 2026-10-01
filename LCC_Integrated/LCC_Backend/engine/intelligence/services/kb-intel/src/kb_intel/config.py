"""kb-intel settings."""

from __future__ import annotations

from typing import Literal

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="KB_INTEL_", extra="ignore")

    service_name: str = "kb-intel"
    service_version: str = "0.1.0"
    environment: Literal["local", "dev", "staging", "production"] = "local"
    log_level: str = "info"
    http_port: int = 8091

    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"
    redis_url: str = "redis://redis:6379"
    vector_db_url: str = "http://qdrant:6333"

    embedding_model: str = "all-MiniLM-L6-v2"
    dedup_min_similarity: float = 0.92
    chunk_max_chars: int = 1200
