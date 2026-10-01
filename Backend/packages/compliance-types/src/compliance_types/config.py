"""ComplianceConfig Python mirror — mirrors `crates/compliance::config`.

Loaded at service start; mutation is forbidden at runtime (axiom 4)."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml
from pydantic import BaseModel, ConfigDict, Field, field_validator


class ActionCaps(BaseModel):
    """Per-action daily caps."""

    model_config = ConfigDict(frozen=True)

    connection_request: int = 18
    direct_message: int = 25
    comment: int = 15
    like: int = 40
    profile_edit_submission: int = 3
    post_publish: int = 3


class Spacing(BaseModel):
    """Minimum inter-action spacing, in seconds."""

    model_config = ConfigDict(frozen=True)

    connection_request: int = 90
    direct_message: int = 60
    comment: int = 45
    like: int = 15


class RiskTierThresholds(BaseModel):
    """RiskTier minimum H_c thresholds."""

    model_config = ConfigDict(frozen=True)

    tier_1: float = 0.0
    tier_2: float = 0.3
    tier_3: float = 0.4
    tier_4: float = 0.5
    tier_5: float = 0.5


DEFAULT_VERSION = "ccfg-2026-04-12-r3"


class ComplianceConfig(BaseModel):
    """Full compliance config payload — versioned, loaded at start."""

    model_config = ConfigDict(frozen=True)

    version: str = DEFAULT_VERSION

    # H_c weights — must sum to 1.0 (validated below)
    h_c_weights: tuple[float, float, float, float] = (0.35, 0.25, 0.20, 0.20)

    caps: ActionCaps = Field(default_factory=ActionCaps)
    warm_up_floor_caps: ActionCaps = Field(
        default_factory=lambda: ActionCaps(
            connection_request=5,
            direct_message=6,
            comment=4,
            like=10,
            profile_edit_submission=1,
            post_publish=1,
        )
    )
    min_spacing_seconds: Spacing = Field(default_factory=Spacing)

    phi_weights: tuple[float, float, float, float] = (0.40, 0.25, 0.20, 0.15)

    reserve_fraction: float = 0.15
    risk_tier_min_h_c: RiskTierThresholds = Field(default_factory=RiskTierThresholds)
    duplicate_target_cooldown_days: int = 30
    session_jitter_min_seconds: int = 8
    phi_qualification_threshold: float = 0.55
    permit_token_ttl_seconds: int = 60
    h_c_warmup_threshold: float = 0.4
    h_c_standard_threshold: float = 0.7
    ab_d_multiplier_floor: float = 0.3
    api_supported_set: list[str] = Field(
        default_factory=lambda: [
            "organization_page_post_publish",
            "oauth_profile_read",
            "jobs_data_read",
        ]
    )

    @field_validator("h_c_weights")
    @classmethod
    def _validate_h_c_weights(cls, v: tuple[float, ...]) -> tuple[float, ...]:
        if len(v) != 4:
            raise ValueError("h_c_weights must have exactly 4 components")
        if abs(sum(v) - 1.0) > 1e-6:
            raise ValueError(f"h_c_weights must sum to 1.0 within 1e-6; got {sum(v)}")
        for i, w in enumerate(v):
            if not (0.0 < w <= 1.0):
                raise ValueError(f"h_c_weights[{i}] = {w} not in (0, 1]")
        return v

    @field_validator("phi_weights")
    @classmethod
    def _validate_phi_weights(cls, v: tuple[float, ...]) -> tuple[float, ...]:
        if len(v) != 4:
            raise ValueError("phi_weights must have exactly 4 components")
        if abs(sum(v) - 1.0) > 1e-6:
            raise ValueError(f"phi_weights must sum to 1.0 within 1e-6; got {sum(v)}")
        return v

    @classmethod
    def from_yaml_file(cls, path: Path | str) -> ComplianceConfig:
        """Load from a YAML file. Validates on load."""
        with open(path, encoding="utf-8") as f:
            data: dict[str, Any] = yaml.safe_load(f)
        return cls(**data)

    @classmethod
    def from_yaml_string(cls, s: str) -> ComplianceConfig:
        """Load from a YAML string. Validates on load."""
        data: dict[str, Any] = yaml.safe_load(s)
        return cls(**data)
