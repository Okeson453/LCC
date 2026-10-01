"""Compliance simulator entrypoint.

Iterates over input scenarios in `inputs/`, exercises the running compliance
governor, and writes results to `reports/`.

Run:
    uv run python tests/compliance-sim/simulator/main.py
"""

from __future__ import annotations

import json
import os
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import httpx

ROOT = Path(__file__).resolve().parent.parent.parent
INPUTS_DIR = ROOT / "tests/compliance-sim/inputs"
REPORTS_DIR = ROOT / "tests/compliance-sim/reports"
RUNS_DIR = ROOT / "tests/compliance-sim/runs"

GOVERNOR_URL = os.environ.get("COMPLIANCE_GOVERNOR_URL", "http://localhost:8080")


@dataclass
class SimResult:
    case_name: str
    expected: str
    actual: str
    pass_: bool
    latency_ms: int
    guards_passed: list[str]
    guards_failed: list[str]


def load_inputs() -> list[dict]:
    cases = []
    for path in sorted(INPUTS_DIR.glob("*.json")):
        cases.append(json.loads(path.read_text()))
    return cases


def evaluate_one(case: dict, client: httpx.Client) -> SimResult:
    body = {
        "member_id": case["member_id"],
        "action_type": case["action_type"],
        "target_kind": case["target_kind"],
        "target_id": case.get("target_id"),
        "context": case.get("context", {}),
    }
    start = time.monotonic()
    try:
        resp = client.post(
            f"{GOVERNOR_URL}/v1/admin/governor/evaluate",
            json=body,
            timeout=10.0,
        )
        latency_ms = int((time.monotonic() - start) * 1000)
        if resp.status_code != 200:
            return SimResult(
                case_name=case["name"],
                expected=case["expected"],
                actual="error",
                pass_=False,
                latency_ms=latency_ms,
                guards_passed=[],
                guards_failed=[],
            )
        data = resp.json()
        actual = data["decision"]
        return SimResult(
            case_name=case["name"],
            expected=case["expected"],
            actual=actual,
            pass_=(actual == case["expected"]),
            latency_ms=latency_ms,
            guards_passed=data.get("guards_passed", []),
            guards_failed=data.get("guards_failed", []),
        )
    except httpx.HTTPError as e:
        return SimResult(
            case_name=case["name"],
            expected=case["expected"],
            actual="exception",
            pass_=False,
            latency_ms=int((time.monotonic() - start) * 1000),
            guards_passed=[],
            guards_failed=[str(e)],
        )


def run() -> int:
    cases = load_inputs()
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)
    RUNS_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = time.strftime("%Y%m%d-%H%M%S")
    results: list[SimResult] = []
    with httpx.Client() as client:
        for case in cases:
            r = evaluate_one(case, client)
            results.append(r)
    pass_count = sum(1 for r in results if r.pass_)
    report = {
        "timestamp": timestamp,
        "total": len(results),
        "passed": pass_count,
        "failed": len(results) - pass_count,
        "results": [asdict(r) for r in results],
    }
    out = REPORTS_DIR / f"sim_report_{timestamp}.json"
    out.write_text(json.dumps(report, indent=2))
    print(f"Compliance simulation: {pass_count}/{len(results)} passed")
    print(f"Report: {out}")
    return 0 if pass_count == len(results) else 1


if __name__ == "__main__":
    raise SystemExit(run())
