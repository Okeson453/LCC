"""ActionType + RiskTier — Python mirror of `crates/compliance::action`.

Mirrors `lcc.v1.compliance.ActionType` in proto.

Per Source Technical Design Spec §3 (axiom 9): numeric thresholds are versioned
config, not literals. A threshold change is a config deploy with its own
audit entry, never a code edit.
"""

from __future__ import annotations

from enum import Enum


class ActionType(str, Enum):
    """Every externally-visible or reputationally-relevant action.

    MUST match the Rust `ActionType` enum in `crates/compliance/src/action.rs`.
    """

    # Tier 1 — drafting / editing (auto-execute)
    PROFILE_EDIT_DRAFT = "profile_edit_draft"
    CONTENT_DRAFT = "content_draft"
    KB_RECORD_CREATE = "kb_record_create"
    SEQUENCE_DRAFT = "sequence_draft"
    OPPORTUNITY_DISCOVER = "opportunity_discover"
    RESEARCH_SCAN = "research_scan"
    VOICE_TRAIN = "voice_train"

    # Tier 2 — light engagement / publish (H_c ≥ 0.3)
    COMMENT = "comment"
    LIKE = "like"
    POST_PUBLISH = "post_publish"
    PROFILE_EDIT_SUBMIT = "profile_edit_submit"

    # Tier 3 — network growth (H_c ≥ 0.4)
    CONNECTION_REQUEST = "connection_request"

    # Tier 4 — 1:1 outreach (H_c ≥ 0.5)
    DIRECT_MESSAGE = "direct_message"
    SEQUENCE_STEP_SEND = "sequence_step_send"

    # Tier 5 — high stakes (H_c ≥ 0.5)
    JOB_APPLICATION_SUBMIT = "job_application_submit"
    CLIENT_PROPOSAL_SEND = "client_proposal_send"
    EXECUTIVE_OUTREACH = "executive_outreach"


ALL_ACTION_TYPES = tuple(ActionType)


_TIER_MAP: dict[ActionType, "RiskTier"] = {
    ActionType.PROFILE_EDIT_DRAFT: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.CONTENT_DRAFT: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.KB_RECORD_CREATE: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.SEQUENCE_DRAFT: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.OPPORTUNITY_DISCOVER: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.RESEARCH_SCAN: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.VOICE_TRAIN: RiskTier.TIER_1_DRAFT_OR_EDIT,
    ActionType.COMMENT: RiskTier.TIER_2_LIGHT_ENGAGEMENT,
    ActionType.LIKE: RiskTier.TIER_2_LIGHT_ENGAGEMENT,
    ActionType.POST_PUBLISH: RiskTier.TIER_2_LIGHT_ENGAGEMENT,
    ActionType.PROFILE_EDIT_SUBMIT: RiskTier.TIER_2_LIGHT_ENGAGEMENT,
    ActionType.CONNECTION_REQUEST: RiskTier.TIER_3_NETWORK_GROWTH,
    ActionType.DIRECT_MESSAGE: RiskTier.TIER_4_ONE_TO_ONE_OUTREACH,
    ActionType.SEQUENCE_STEP_SEND: RiskTier.TIER_4_ONE_TO_ONE_OUTREACH,
    ActionType.JOB_APPLICATION_SUBMIT: RiskTier.TIER_5_HIGH_STAKES,
    ActionType.CLIENT_PROPOSAL_SEND: RiskTier.TIER_5_HIGH_STAKES,
    ActionType.EXECUTIVE_OUTREACH: RiskTier.TIER_5_HIGH_STAKES,
}


class RiskTier(int, Enum):
    """RiskTier (1..5) — required minimum H_c to permit the action."""

    TIER_1_DRAFT_OR_EDIT = 1
    TIER_2_LIGHT_ENGAGEMENT = 2
    TIER_3_NETWORK_GROWTH = 3
    TIER_4_ONE_TO_ONE_OUTREACH = 4
    TIER_5_HIGH_STAKES = 5

    @property
    def minimum_h_c(self) -> float:
        thresholds = {
            RiskTier.TIER_1_DRAFT_OR_EDIT: 0.0,
            RiskTier.TIER_2_LIGHT_ENGAGEMENT: 0.3,
            RiskTier.TIER_3_NETWORK_GROWTH: 0.4,
            RiskTier.TIER_4_ONE_TO_ONE_OUTREACH: 0.5,
            RiskTier.TIER_5_HIGH_STAKES: 0.5,
        }
        return thresholds[self]


def action_risk_tier(action_type: ActionType) -> RiskTier:
    """Return the RiskTier for this ActionType."""
    return _TIER_MAP[action_type]
