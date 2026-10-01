"""Cross-validate the Python mirror of ComplianceConfig against the Rust crate.

Runs at CI time (`tools/scripts/check_config_parity.py`) — fails the build if the
two drift apart on default values or required fields.

This is a structural check, not a runtime guarantee. It exists so a change to
either side forces the other side to be re-checked.
"""

from __future__ import annotations

from .config import (
    ComplianceConfig,
    ActionCaps,
    Spacing,
    RiskTierThresholds,
    DEFAULT_VERSION,
)


# Mirror of `crates/compliance::config::ActionCaps::warm_up_floor` / `Default`.
WARM_UP_FLOOR = {
    "connection_request": 5,
    "direct_message": 6,
    "comment": 4,
    "like": 10,
    "profile_edit_submission": 1,
    "post_publish": 1,
}

STANDARD_CAPS = {
    "connection_request": 18,
    "direct_message": 25,
    "comment": 15,
    "like": 40,
    "profile_edit_submission": 3,
    "post_publish": 3,
}

DEFAULT_SPACING = {
    "connection_request": 90,
    "direct_message": 60,
    "comment": 45,
    "like": 15,
}

DEFAULT_RISK_TIER_MIN_H_C = {
    "tier_1": 0.0,
    "tier_2": 0.3,
    "tier_3": 0.4,
    "tier_4": 0.5,
    "tier_5": 0.5,
}


def assert_rust_python_parity() -> None:
    """Raise AssertionError if the Python defaults drift from the Rust defaults.

    The Rust defaults are encoded here as a single source of truth — that is, the
    Rust maintainer edits this file when changing `crates/compliance::config`.
    """
    cfg = ComplianceConfig()

    # Version
    assert cfg.version == DEFAULT_VERSION == "ccfg-2026-04-12-r3", (
        f"version drift: got {cfg.version}"
    )

    # H_c weights
    assert cfg.h_c_weights == (0.35, 0.25, 0.20, 0.20), (
        f"h_c_weights drift: got {cfg.h_c_weights}"
    )

    # Phi weights
    assert cfg.phi_weights == (0.40, 0.25, 0.20, 0.15), (
        f"phi_weights drift: got {cfg.phi_weights}"
    )

    # Standard caps
    for k, v in STANDARD_CAPS.items():
        assert getattr(cfg.caps, k) == v, f"caps.{k}: expected {v}, got {getattr(cfg.caps, k)}"

    # Warm-up floor caps
    for k, v in WARM_UP_FLOOR.items():
        assert getattr(cfg.warm_up_floor_caps, k) == v, (
            f"warm_up_floor_caps.{k}: expected {v}, got {getattr(cfg.warm_up_floor_caps, k)}"
        )

    # Spacing
    for k, v in DEFAULT_SPACING.items():
        assert getattr(cfg.min_spacing_seconds, k) == v, (
            f"min_spacing_seconds.{k}: expected {v}, got {getattr(cfg.min_spacing_seconds, k)}"
        )

    # Reserve fraction
    assert cfg.reserve_fraction == 0.15, f"reserve_fraction: got {cfg.reserve_fraction}"

    # Thresholds
    for k, v in DEFAULT_RISK_TIER_MIN_H_C.items():
        assert getattr(cfg.risk_tier_min_h_c, k) == v, (
            f"risk_tier_min_h_c.{k}: expected {v}, got {getattr(cfg.risk_tier_min_h_c, k)}"
        )

    assert cfg.h_c_warmup_threshold == 0.4, "h_c_warmup_threshold drift"
    assert cfg.h_c_standard_threshold == 0.7, "h_c_standard_threshold drift"
    assert cfg.ab_d_multiplier_floor == 0.3, "ab_d_multiplier_floor drift"
    assert cfg.permit_token_ttl_seconds == 60, "permit_token_ttl_seconds drift"
    assert cfg.phi_qualification_threshold == 0.55, "phi_qualification_threshold drift"
    assert cfg.duplicate_target_cooldown_days == 30, "duplicate_target_cooldown_days drift"
    assert cfg.session_jitter_min_seconds == 8, "session_jitter_min_seconds drift"
