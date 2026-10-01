"""Model loader for rho (mirrors opportunity-intel's loader).

F-AUDIT-25 — honesty of `mode`:
    This loader previously returned a hardcoded `RhoModelArtifact` stamped
    `version="0.0.0-default"` when no trained artifact was on disk, and
    `RhoModelArtifact.predict` unconditionally reported `mode="ml"`. Combined
    with `rho.py`'s `labeled_sends >= 200` gate, that meant that as soon as a
    member accumulated 200 sends, the service started reporting an ML model
    while in fact running the same hardcoded constants — with no training
    anywhere in the repository. This is precisely the failure the design
    forbids (Technical Design Spec §5: beta coefficients "are NOT shipped
    with production defaults ... until that sample size is reached, the system
    falls back to a rule-based priority order, stated explicitly rather than
    substituting an uncalibrated model silently").

    The loader now returns `None` when no *trained* artifact exists, so the
    caller falls back to the rule-based scorer and labels the result
    `rule_based`. `mode="ml"` is now only ever reachable with a real,
    versioned, fitted artifact on disk.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path

from scoring_intel.core.rho import ContactFeature, RhoScore

_MODEL_DIR = Path(os.environ.get("LCC_MODEL_DIR", "/tmp/lcc-models"))


@dataclass
class RhoModelArtifact:
    version: str
    weights: dict[str, float]
    #: Number of labeled samples the artifact was fitted on. Used to reject an
    #: artifact that claims to be trained but was not fit on enough data.
    fitted_on_samples: int = 0

    def predict(self, contact: ContactFeature) -> RhoScore:
        # Toy ensemble using the artifact's weights.
        w = self.weights
        recency = max(0.0, 1.0 - contact.last_contact_days / 30.0)
        kb_fit = min(1.0, contact.matched_kb_facts / 5.0)
        vip = 1.0 if contact.is_vip else 0.0
        mutual = 1.0 if contact.is_mutual else 0.0
        score = max(
            0.0,
            min(
                1.0,
                w.get("vip", 0.3) * vip
                + w.get("recency", 0.3) * recency
                + w.get("mutual", 0.1) * mutual
                + w.get("kb_fit", 0.15) * kb_fit
                + w.get("response_history", 0.15) * contact.response_rate,
            ),
        )
        return RhoScore(
            contact_id=contact.contact_id,
            score=score,
            components={
                "vip": vip,
                "recency": recency,
                "mutual": mutual,
                "kb_fit": kb_fit,
                "response_history": contact.response_rate,
            },
            mode="ml",
            rationale=[f"rho model version {self.version}"],
        )


_rho_model: RhoModelArtifact | None = None


#: Minimum labeled samples an artifact must have been fitted on before it is
#: allowed to drive prioritisation. Matches Technical Design Spec §5 and
#: §29 ("Reply-probability model (rho) fit | >=200 labeled sends").
MIN_FITTED_SAMPLES = 200

#: Sentinel version identifying the absence of a trained model.
NO_MODEL_VERSION = "none"


def load_rho_model() -> RhoModelArtifact | None:
    """Load the trained rho artifact, or return ``None`` if there isn't one.

    Returning ``None`` — rather than a default-weights artifact — is what lets
    the caller stay honest: no fitted model means rule-based scoring with
    ``mode="rule_based"``, not an ML mode backed by magic numbers.
    """
    global _rho_model
    if _rho_model is not None:
        return _rho_model or None

    model_path = _MODEL_DIR / "rho_model.json"
    if not model_path.exists():
        _rho_model = None
        return None

    try:
        data = json.loads(model_path.read_text())
    except (OSError, ValueError):
        # A corrupt artifact must not be mistaken for a trained one.
        _rho_model = None
        return None

    fitted_on = int(data.get("fitted_on_samples", 0))
    if fitted_on < MIN_FITTED_SAMPLES:
        _rho_model = None
        return None

    _rho_model = RhoModelArtifact(
        version=data.get("version", NO_MODEL_VERSION),
        weights=data.get("weights", {}),
        fitted_on_samples=fitted_on,
    )
    return _rho_model


__all__ = [
    "load_rho_model",
    "RhoModelArtifact",
    "MIN_FITTED_SAMPLES",
    "NO_MODEL_VERSION",
]
