"""scoring-intel settings."""

from __future__ import annotations

from typing import Literal

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_prefix="SCORING_INTEL_", extra="ignore")

    service_name: str = "scoring-intel"
    service_version: str = "0.1.0"
    environment: Literal["local", "dev", "staging", "production"] = "local"
    log_level: str = "info"
    http_port: int = 8093

    postgres_url: str = "postgresql://postgres:postgres@postgres:5432/lcc"
    redis_url: str = "redis://redis:6379"

    # ρ thresholds
    rho_labeled_send_threshold: int = 200
    rho_vip_boost: float = 1.5

    # h_c defaults
    hc_warmup_threshold: float = 0.85
    hc_standard_threshold: float = 0.65
    hc_reserve_fraction: float = 0.10

    # ab_d
    abd_multiplier_floor: float = 0.5
    abd_multiplier_ceiling: float = 2.0
