"""Action Budget (AB_d) — Python mirror of crates/compliance::ab_d."""

from __future__ import annotations

from compliance_types.action import ActionType
from compliance_types.config import ComplianceConfig


def compute_ab_d(
    action_type: ActionType,
    h_c: float,
    config: ComplianceConfig,
) -> int:
    """Compute AB_d for the given action and account health.

    Source: Technical Design Spec §4.
    H_c is clamped to [0,1]. The multiplier is the linear interpolation
    between `ab_d_multiplier_floor` (H_c=0) and 1.0 (H_c=1).
    """
    h_c_clamped = 0.0 if (h_c != h_c) else max(0.0, min(1.0, h_c))

    cap_standard = config.caps.connection_request if action_type == ActionType.CONNECTION_REQUEST else 0
    cap_warmup = config.warm_up_floor_caps.connection_request if action_type == ActionType.CONNECTION_REQUEST else 0

    if action_type == ActionType.DIRECT_MESSAGE or action_type == ActionType.SEQUENCE_STEP_SEND:
        cap_standard = config.caps.direct_message
        cap_warmup = config.warm_up_floor_caps.direct_message
    elif action_type == ActionType.COMMENT:
        cap_standard = config.caps.comment
        cap_warmup = config.warm_up_floor_caps.comment
    elif action_type == ActionType.LIKE:
        cap_standard = config.caps.like
        cap_warmup = config.warm_up_floor_caps.like
    elif action_type == ActionType.PROFILE_EDIT_SUBMIT:
        cap_standard = config.caps.profile_edit_submission
        cap_warmup = config.warm_up_floor_caps.profile_edit_submission
    elif action_type == ActionType.POST_PUBLISH:
        cap_standard = config.caps.post_publish
        cap_warmup = config.warm_up_floor_caps.post_publish

    if cap_standard == 0:
        return 0

    if h_c_clamped <= config.h_c_warmup_threshold:
        cap = cap_warmup
    elif h_c_clamped >= config.h_c_standard_threshold:
        cap = cap_standard
    else:
        t = (h_c_clamped - config.h_c_warmup_threshold) / (
            config.h_c_standard_threshold - config.h_c_warmup_threshold
        )
        cap_w = float(cap_warmup)
        cap_s = float(cap_standard)
        cap = int(cap_w + (cap_s - cap_w) * t)

    multiplier = config.ab_d_multiplier_floor + (1.0 - config.ab_d_multiplier_floor) * h_c_clamped
    multiplier_clamped = max(config.ab_d_multiplier_floor, min(1.0, multiplier))
    return int(cap * multiplier_clamped)
