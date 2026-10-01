"""Compliance simulation: run a sequence of synthetic actions through
the governor and verify the expected deny/allow decisions.

Runs against the compliance-governor service at $COMPLIANCE_GOVERNOR_URL.
"""

import os
import json
from dataclasses import dataclass


@dataclass
class SimCase:
    name: str
    member_id: str
    action_type: str
    target_kind: str
    expected: str  # 'allow' | 'deny'


CASES = [
    SimCase("first_connection_request", "00000000-0000-0000-0000-000000000001",
            "connection_request", "connection", "allow"),
    SimCase("connection_to_same_target_within_cooldown", "00000000-0000-0000-0000-000000000001",
            "connection_request", "connection", "deny"),
    SimCase("daily_cap_exceeded", "00000000-0000-0000-0000-000000000001",
            "connection_request", "connection", "deny"),
    SimCase("post_publish_with_low_grounding", "00000000-0000-0000-0000-000000000001",
            "post_publish", "post", "deny"),
    SimCase("dm_with_h_c_below_standard", "00000000-0000-0000-0000-000000000001",
            "dm", "dm", "deny"),
]


def run_simulation():
    """In production this is wired to the governor. Stub for now."""
    results = []
    for case in CASES:
        # POST to governor → verify decision == case.expected
        results.append({"name": case.name, "expected": case.expected, "actual": "stub"})
    return results


if __name__ == "__main__":
    print(json.dumps(run_simulation(), indent=2))
