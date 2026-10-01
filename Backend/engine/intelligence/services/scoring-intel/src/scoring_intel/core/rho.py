"""ρ (rho) scorer — contact priority for sequencing.

Per Backend Design Concept §50, ρ ranks contacts for sequence selection:
- ML model when labeled_sends >= 200.
- Rule-based fallback: VIP weight > recency > mutual connections > KB fit.

Returns a `RhoScore` (score, mode, rationale).
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ContactFeature:
    contact_id: str
    is_vip: bool = False
    is_mutual: bool = False
    last_contact_days: int = 365
    matched_kb_facts: int = 0
    response_rate: float = 0.0  # 0..1, from prior inbound history
    accept_rate: float = 0.0     # 0..1


@dataclass
class RhoScore:
    contact_id: str
    score: float          # 0..1
    components: dict[str, float]
    mode: str             # "ml" | "rule_based"
    rationale: list[str]


def score_contact(
    contact: ContactFeature,
    *,
    labeled_sends: int,
    vip_boost: float = 1.5,
) -> RhoScore:
    if labeled_sends >= 200:
        return _score_ml(contact)
    return _score_rule_based(contact, vip_boost=vip_boost)


def _score_rule_based(
    contact: ContactFeature,
    vip_boost: float,
    extra_rationale: list[str] | None = None,
) -> RhoScore:
    components = {
        "vip": 0.0,
        "recency": 0.0,
        "mutual": 0.0,
        "kb_fit": 0.0,
        "response_history": contact.response_rate,
    }
    rationale = []

    # VIP weight: 0.5 + vip_boost * 0.5
    if contact.is_vip:
        components["vip"] = min(1.0, 0.5 * vip_boost)
        rationale.append("VIP boost applied")

    # Recency: inverse of days (0..1 over 30-day window)
    recency = max(0.0, 1.0 - contact.last_contact_days / 30.0)
    components["recency"] = recency

    # Mutual: 0.2 boost
    if contact.is_mutual:
        components["mutual"] = 0.2
        rationale.append("mutual connection")

    # KB fit: 0..1 over 5 facts
    kb_fit = min(1.0, contact.matched_kb_facts / 5.0)
    components["kb_fit"] = kb_fit

    # Linear ensemble.
    weights = {"vip": 0.35, "recency": 0.30, "mutual": 0.10, "kb_fit": 0.15, "response_history": 0.10}
    raw = sum(components[k] * w for k, w in weights.items())
    score = max(0.0, min(1.0, raw))

    if extra_rationale:
        rationale.extend(extra_rationale)

    return RhoScore(
        contact_id=contact.contact_id,
        score=score,
        components=components,
        mode="rule_based",
        rationale=rationale,
    )


def _score_ml(contact: ContactFeature) -> RhoScore:
    """Use the fitted rho model, or degrade honestly to rule-based.

    F-AUDIT-25: this previously called `load_rho_model()` and used its result
    unconditionally. The loader always returned *something*, so this returned
    `mode="ml"` on top of hardcoded default weights the moment a member hit
    200 labeled sends. Now the loader returns ``None`` unless a genuinely
    fitted artifact exists, and we fall back to the rule-based scorer —
    labelled as such — so a consumer gating on `mode` (e.g. the
    `rule_based_fallback` flag in `proto/lcc/v1/intelligence/scoring.proto`)
    is never told a model is live when it is not.
    """
    try:
        from scoring_intel.core.model_loader import load_rho_model

        model = load_rho_model()
        if model is not None:
            return model.predict(contact)
        return _score_rule_based(
            contact,
            vip_boost=1.5,
            extra_rationale=[
                "labeled_sends >= 200 but no fitted rho artifact is available; "
                "using the documented rule-based order (VIP > recency > mutual)"
            ],
        )
    except Exception:  # pragma: no cover - defensive
        return _score_rule_based(
            contact,
            vip_boost=1.5,
            extra_rationale=["rho model load failed; using rule-based order"],
        )


__all__ = ["ContactFeature", "RhoScore", "score_contact"]
