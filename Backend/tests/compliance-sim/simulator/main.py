"""Compliance simulator entrypoint.

Drives the running compliance-governor with the scenarios in `inputs/` and
fails when a decision disagrees with the scenario's expectation.

Run:
    python tests/compliance-sim/simulator/main.py

Environment:
    COMPLIANCE_GOVERNOR_URL   base URL of the governor (default http://localhost:8080)
    COMPLIANCE_CONFIG_VERSION active config version the governor loaded
                              (default: derived from COMPLIANCE_CONFIG_PATH)

Each input file is one scenario:

    {
      "name": "deny_grounding_missing",
      "action_type": "post_publish",       # ActionType, snake_case
      "target_id": "post-1",               # optional, stable per scenario
      "expected": "DENY",                  # PERMIT | DENY
      "expected_guard": "grounding",        # optional: failed_guard assertion
      "requires_approval": false,          # optional, default true
      "auto_execute": false,               # optional, default false
      "kb_refs": 1,                        # optional, number of cited KB records
      "account": {                         # optional account snapshot overrides
        "h_c": 0.8,
        "warmup_days_remaining": 0,
        "days_active": 90,
        "is_restricted": false,
        "restricted_reason": null
      }
    }

The request is the governor's real `EvaluateActionRequestDto`: the scenario is
expanded into a candidate action, an account snapshot and a trace id. A
scenario passes only when the decision matches, and — when `expected_guard` is
present — the governor denied on that specific guard, so a scenario cannot pass
for the wrong reason.
"""

from __future__ import annotations

import json
import os
import sys
import time
import uuid
from dataclasses import asdict, dataclass
from pathlib import Path

import httpx

# .../Backend/tests/compliance-sim/simulator/main.py -> .../Backend
BACKEND_ROOT = Path(__file__).resolve().parents[3]
SIM_ROOT = BACKEND_ROOT / "tests" / "compliance-sim"
INPUTS_DIR = SIM_ROOT / "inputs"
REPORTS_DIR = SIM_ROOT / "reports"

GOVERNOR_URL = os.environ.get("COMPLIANCE_GOVERNOR_URL", "http://localhost:8080")
# The internal route is the unauthenticated legacy alias of the canonical
# `/api/v1/admin/evaluate` (which requires an admin JWT). The simulator has no
# credential by design: it must not be able to change compliance config.
EVALUATE_PATH = "/internal/governor/evaluate"

#: Member the scenarios act as. Fixed so scenarios in one run share account
#: history and the audit trail is readable.
MEMBER_ID = "00000000-0000-0000-0000-000000000001"

#: Guard names emitted by the Rust governor, for `expected_guard` validation.
KNOWN_GUARDS = {
    "daily_cap",
    "cooldown",
    "duplicate_target",
    "account_health",
    "grounding",
    "restriction_flag",
    "approval_state",
    "session_pacing",
}


def _config_version() -> str:
    """Version string of the compliance config the governor loaded."""
    explicit = os.environ.get("COMPLIANCE_CONFIG_VERSION")
    if explicit:
        return explicit
    path = os.environ.get("COMPLIANCE_CONFIG_PATH", "")
    if path:
        return Path(path).stem
    return "ccfg-2025-01-01-rc1"


def _risk_tier_for(action_type: str) -> str:
    """Risk tier for an action type.

    Uses the Python mirror of the Rust taxonomy (`compliance-types`) so the
    scenario files only name the action, never a hand-maintained tier.
    """
    try:
        from compliance_types.action import ActionType
    except ImportError:  # pragma: no cover - path issue, not a scenario issue
        raise SystemExit(
            "compliance_types not importable; run with PYTHONPATH including "
            "Backend/packages/compliance-types/src"
        ) from None
    try:
        return ActionType(action_type).risk_tier().name.lower()
    except ValueError:
        raise SystemExit(f"unknown action_type in scenario: {action_type}") from None


@dataclass
class SimResult:
    case_name: str
    expected: str
    actual: str
    pass_: bool
    failed_guard: str
    reason: str
    latency_ms: int


def load_inputs() -> list[dict]:
    return [json.loads(p.read_text()) for p in sorted(INPUTS_DIR.glob("*.json"))]


def build_request(case: dict) -> dict:
    """Expand a scenario into the governor's EvaluateActionRequestDto."""
    account = {
        "h_c": 0.8,
        "warmup_days_remaining": 0,
        "days_active": 90,
        "is_restricted": False,
        "restricted_reason": None,
    }
    account.update(case.get("account", {}))
    kb_ref_count = int(case.get("kb_refs", 1))
    target = case.get("target_id")
    target_kind = case.get("target_kind", "post")

    # `target_contact_id` drives the duplicate-target guard, so only the
    # contact-shaped targets are attached as a UUID.
    target_contact_id = None
    if target and target_kind == "contact":
        target_contact_id = str(uuid.uuid5(uuid.NAMESPACE_URL, f"lcc-sim/contact/{target}"))

    return {
        "candidate_action": {
            "id": str(uuid.uuid5(uuid.NAMESPACE_URL, f"lcc-sim/action/{case['name']}")),
            "action_type": case["action_type"],
            "member_id": MEMBER_ID,
            "risk_tier": _risk_tier_for(case["action_type"]),
            "kb_refs": [
                str(uuid.uuid5(uuid.NAMESPACE_URL, f"lcc-sim/kb/{case['name']}/{i}"))
                for i in range(kb_ref_count)
            ],
            "idempotency_key": f"sim-{case['name']}",
            "requires_approval": bool(case.get("requires_approval", True)),
            "auto_execute": bool(case.get("auto_execute", False)),
            "target_contact_id": target_contact_id,
            "opportunity_id": None,
            "content_item_id": None,
            "sequence_step_id": None,
            "application_id": None,
            "metadata": {"sim_case": case["name"]},
        },
        "account_state": {
            "h_c": float(account["h_c"]),
            # Fresh timestamp: a stale value makes the account-health guard call
            # scoring-intel, which this evaluation deliberately does not run.
            "h_c_computed_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "is_restricted": bool(account["is_restricted"]),
            "restricted_since": account.get("restricted_since"),
            "restricted_reason": account.get("restricted_reason"),
            "warmup_days_remaining": int(account["warmup_days_remaining"]),
            "days_active": int(account["days_active"]),
            "active_compliance_config_version": _config_version(),
        },
        "trace_id": f"sim-{case['name']}",
    }


def evaluate_one(case: dict, client: httpx.Client) -> SimResult:
    body = build_request(case)
    start = time.monotonic()
    try:
        resp = client.post(f"{GOVERNOR_URL}{EVALUATE_PATH}", json=body, timeout=15.0)
    except httpx.HTTPError as e:
        return SimResult(
            case_name=case["name"],
            expected=case["expected"],
            actual=f"transport_error: {e}",
            pass_=False,
            failed_guard="",
            reason="",
            latency_ms=int((time.monotonic() - start) * 1000),
        )
    latency_ms = int((time.monotonic() - start) * 1000)

    # The governor answers 200 for PERMIT and 403 for DENY; anything else is a
    # harness error rather than a compliance decision.
    if resp.status_code not in (200, 403):
        return SimResult(
            case_name=case["name"],
            expected=case["expected"],
            actual=f"http_{resp.status_code}",
            pass_=False,
            failed_guard="",
            reason=resp.text[:300],
            latency_ms=latency_ms,
        )

    data = resp.json()
    actual = str(data.get("decision", "?"))
    failed_guard = str(data.get("failed_guard") or "")
    reason = str(data.get("reason") or "")

    ok = actual == case["expected"]
    if ok and case.get("expected_guard"):
        expected_guard = case["expected_guard"]
        if expected_guard not in KNOWN_GUARDS:
            raise SystemExit(f"scenario {case['name']} names unknown guard: {expected_guard}")
        # A DENY for the wrong guard is still a broken guard, so require the
        # specific one the scenario is about.
        ok = failed_guard == expected_guard

    return SimResult(
        case_name=case["name"],
        expected=case["expected"],
        actual=actual if ok else f"{actual} ({failed_guard or 'no guard'})",
        pass_=ok,
        failed_guard=failed_guard,
        reason=reason,
        latency_ms=latency_ms,
    )


def run() -> int:
    cases = load_inputs()
    if not cases:
        print(f"no scenarios found in {INPUTS_DIR}")
        return 1

    results: list[SimResult] = []
    with httpx.Client() as client:
        # Fail fast and loudly when the governor is not up: an unreachable
        # governor must not be reported as a compliance result.
        try:
            client.get(f"{GOVERNOR_URL}/readyz", timeout=5.0)
        except httpx.HTTPError as e:
            print(f"compliance-governor not reachable at {GOVERNOR_URL}: {e}")
            return 1
        for case in cases:
            results.append(evaluate_one(case, client))

    passed = sum(1 for r in results if r.pass_)
    print(f"{'case':40} {'expected':8} {'actual':28} result")
    for r in results:
        print(f"{r.case_name:40} {r.expected:8} {r.actual:28} {'PASS' if r.pass_ else 'FAIL'}")
        if not r.pass_ and r.reason:
            print(f"    reason: {r.reason}")
    print(f"\nCompliance simulation: {passed}/{len(results)} passed")

    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = time.strftime("%Y%m%d-%H%M%S")
    report = {
        "timestamp": timestamp,
        "governor_url": GOVERNOR_URL,
        "config_version": _config_version(),
        "total": len(results),
        "passed": passed,
        "failed": len(results) - passed,
        "results": [asdict(r) for r in results],
    }
    out = REPORTS_DIR / f"sim_report_{timestamp}.json"
    out.write_text(json.dumps(report, indent=2))
    print(f"Report: {out}")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(run())
