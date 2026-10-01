"""compliance-types — Python mirror of crates/compliance.

Used by Intelligence Engine services for cross-language type compatibility.
The Rust side remains the source of truth for the underlying math; this
package mirrors the public type surface for use in Python service code.
"""

from compliance_types.action import (
    ActionType,
    RiskTier,
    action_risk_tier,
    ALL_ACTION_TYPES,
)
from compliance_types.config import (
    ComplianceConfig,
    ActionCaps,
    Spacing,
    RiskTierThresholds,
    DEFAULT_VERSION,
)
from compliance_types.hc import (
    HcInputs,
    HcComponents,
    compute_h_c,
    DEFAULT_HC_WEIGHTS,
)
from compliance_types.abd import compute_ab_d
from compliance_types.phi import (
    PhiInputs,
    PhiComponents,
    GoalMode,
    compute_phi,
    DEFAULT_PHI_WEIGHTS,
)
from compliance_types.rho import (
    ReplyProbabilityFeatures,
    RhoMode,
    RhoResult,
    predict_reply_probability,
    MIN_LABELED_SENDS,
)
from compliance_types.permits import PermitClaims

__all__ = [
    "ActionType",
    "RiskTier",
    "action_risk_tier",
    "ALL_ACTION_TYPES",
    "ComplianceConfig",
    "ActionCaps",
    "Spacing",
    "RiskTierThresholds",
    "DEFAULT_VERSION",
    "HcInputs",
    "HcComponents",
    "compute_h_c",
    "DEFAULT_HC_WEIGHTS",
    "compute_ab_d",
    "PhiInputs",
    "PhiComponents",
    "GoalMode",
    "compute_phi",
    "DEFAULT_PHI_WEIGHTS",
    "ReplyProbabilityFeatures",
    "RhoMode",
    "RhoResult",
    "predict_reply_probability",
    "MIN_LABELED_SENDS",
    "PermitClaims",
]
