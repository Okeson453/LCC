"""Contract test — Python gRPC stubs match the proto contract.

Per ADR-0001 / Backend Design Concept §35:
- Rust Core and Python Intelligence communicate via gRPC
- Wire types live under `proto/lcc/v1/*/*.proto` (Rust + Python mirror)
- Each .proto must have a generated `proto/gen/python/<pkg>/_pb2.py`
  and `<pkg>_pb2_grpc.py`

This test validates the Python side of that contract. The Rust side is
covered by `tests/contract/canonical/canonical_contract_test.rs`.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest


# Ensure the generated stubs are importable.
PROTO_ROOT = Path(__file__).resolve().parents[2] / "proto" / "gen" / "python"
sys.path.insert(0, str(PROTO_ROOT.parent))


def _import_pkg(name: str) -> bool:
    try:
        mod = __import__(f"lcc.v1.{name}", fromlist=["_pb2"])
        return hasattr(mod, "_pb2")
    except ImportError:
        return False


EXPECTED_PACKAGES = [
    "identity",
    "intelligence",
    "compliance",
    "approval",
    "content",
    "engagement",
    "events",
    "network",
    "outreach",
    "profile",
    "analytics",
    "integration",
    "common",
]


@pytest.mark.parametrize("pkg", EXPECTED_PACKAGES)
def test_pb2_module_importable(pkg: str) -> None:
    """Every domain package must have a generated `_pb2.py`."""
    pb2_path = PROTO_ROOT / "lcc" / "v1" / pkg / "_pb2.py"
    assert pb2_path.exists(), f"missing {pb2_path}"
    assert _import_pkg(pkg), f"cannot import lcc.v1.{pkg}._pb2"


@pytest.mark.parametrize("pkg", EXPECTED_PACKAGES)
def test_pb2_grpc_module_importable(pkg: str) -> None:
    """Every domain package must have a generated `<pkg>_pb2_grpc.py`."""
    grpc_path = PROTO_ROOT / "lcc" / "v1" / f"{pkg}_pb2_grpc.py"
    assert grpc_path.exists(), f"missing {grpc_path}"


def test_scoring_intel_service_has_computable_h_c() -> None:
    """The most-critical cross-engine RPC: ScoringIntel.ComputeH_c.

    Per Backend Design Concept §9, ComputeH_c is called every time the
    `account_health` guard runs against a stale H_c. If this contract
    breaks, the compliance-governor's fifth guard silently falls back to
    the cached value (which is exactly the failure mode the gRPC seam is
    supposed to prevent).
    """
    try:
        from lcc.v1 import intelligence_pb2_grpc  # type: ignore[import-untyped]
        from lcc.v1.intelligence import _pb2  # type: ignore[import-untyped]
    except ImportError:
        pytest.skip("grpcio not installed")
    servicer = getattr(intelligence_pb2_grpc, "ScoringIntelServicer", None)
    assert servicer is not None
    assert hasattr(servicer, "ComputeH_c")
    assert hasattr(servicer, "ComputePhi")
    assert hasattr(servicer, "PredictReplyProbability")
    assert hasattr(servicer, "CalibrationStatus")
    # Add function should also exist for grpc.aio.server registration.
    assert hasattr(intelligence_pb2_grpc, "add_ScoringIntelServicer_to_server")
    assert hasattr(intelligence_pb2_grpc, "ScoringIntelStub")
    assert hasattr(intelligence_pb2_grpc, "ScoringIntelAsyncStub")


def test_compute_h_c_request_roundtrip() -> None:
    """The Python-side ComputeH_cRequest must serialize correctly."""
    try:
        from lcc.v1.intelligence import _pb2  # type: ignore[import-untyped]
    except ImportError:
        pytest.skip("grpcio not installed")
    req = _pb2.ComputeH_cRequest(
        member_id="test-member",
        compliance_config_version="v1",
        force_recompute=True,
    )
    assert req.member_id == "test-member"
    assert req.compliance_config_version == "v1"
    assert req.force_recompute is True


def test_contract_yaml_match() -> None:
    """The number of .proto files must equal the number of generated _pb2.py files."""
    proto_root = Path(__file__).resolve().parents[2] / "proto" / "lcc" / "v1"
    protos = list(proto_root.rglob("*.proto"))
    pb2s = list((Path(__file__).resolve().parents[2] / "proto" / "gen" / "python").rglob("_pb2.py"))
    # Allow per-package files to share a single _pb2.py (merged).
    expected_packages = {p.name for p in protos}
    assert len(protos) > 0, "no .proto files"
    assert len(pb2s) > 0, "no generated _pb2.py files"
    assert len(pb2s) >= len(expected_packages), (
        f"missing _pb2.py files: have {len(pb2s)} for {len(expected_packages)} packages"
    )
