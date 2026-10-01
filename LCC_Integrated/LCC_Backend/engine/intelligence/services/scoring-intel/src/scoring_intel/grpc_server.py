"""scoring-intel gRPC server.

The Python Intelligence engine's `scoring-intel` service exposes its work
over gRPC (per ADR-0001 — boundary rule: cross-engine calls must use gRPC,
not HTTP). The Rust Core `compliance-governor` connects to this server via
tonic and calls the four `ScoringIntel` RPCs:

  - ComputeH_c          (refresh H_c account-health scalar)
  - ComputePhi          (re-derive φ for an opportunity)
  - PredictReplyProbability (ρ inference)
  - CalibrationStatus   (audit reporting)

The wire types come from the generated Python stubs under
`proto/gen/python/lcc/v1/intelligence/`. The server is a thin shim over the
existing FastAPI handlers (`scoring_intel.api.hc`, `.abd`, `.rho`), so the
business logic is shared.

For local dev / sandboxes that don't have `grpcio` installed, the import
guards at the top of the module degrade gracefully — the gRPC server
simply doesn't start. CI gates on this; if `grpcio` is missing, the build
fails the boundary check.
"""

from __future__ import annotations

import asyncio
import logging
import os
import time
from typing import Optional

logger = logging.getLogger(__name__)

# Graceful degradation: if grpcio is missing, the server logs and exits.
try:
    import grpc  # type: ignore[import-untyped]
    GRPC_AVAILABLE = True
except ImportError:
    GRPC_AVAILABLE = False
    logger.warning("grpcio not installed; scoring-intel gRPC server will not start")

# Generated stubs from `proto/gen/python/codegen.py`. The package is
# checked out under proto/gen/python and added to PYTHONPATH at runtime.
try:
    from lcc.v1 import intelligence_pb2_grpc  # type: ignore[import-untyped]
    from lcc.v1.intelligence import _pb2  # type: ignore[import-untyped]
    PROTO_AVAILABLE = True
except ImportError:
    PROTO_AVAILABLE = False
    logger.warning(
        "lcc.v1.intelligence proto stubs not on PYTHONPATH; "
        "scoring-intel gRPC server will not start. "
        "Run: python3 proto/gen/python/codegen.py --proto proto/lcc/v1 --out proto/gen/python"
    )


# Lazy import of business logic — kept inside the function so the module
# can be imported for unit tests even if the gRPC stack is missing.
async def _compute_h_c_handler(request, context):  # type: ignore[no-untyped-def]
    """Server-side handler for ComputeH_c.

    Delegates to the existing FastAPI handler `hc.compute_h_c` to keep
    business logic in one place. The wrapper translates between the
    protobuf wire types and the HTTP-layer types.
    """
    from scoring_intel.api import hc  # local import to avoid heavy dep at import time
    from scoring_intel.config import Settings

    settings = Settings()
    # Translation layer: protobuf wire -> HTTP body.
    body = {
        "member_id": request.member_id,
        "compliance_config_version": request.compliance_config_version,
        "force_recompute": request.force_recompute,
    }
    http_result = await hc.compute_h_c_for_grpc(body, settings)
    # Translate back: HTTP body -> protobuf wire.
    return _pb2.ComputeH_cResponse(
        result=_pb2.H_cResult(
            h_c=http_result.get("h_c", 0.0),
            components=_pb2.H_cComponents(
                acceptance_rate=http_result.get("acceptance_rate", 0.0),
                reply_rate=http_result.get("reply_rate", 0.0),
                quota_utilization=http_result.get("quota_utilization", 0.0),
                tenure_factor=http_result.get("tenure_factor", 0.0),
                weights=http_result.get("weights", []),
            ),
            computed_at_unix_ms=int(time.time() * 1000),
            h_c_undefined=http_result.get("h_c_undefined", False),
            reason=http_result.get("reason", ""),
        )
    )


async def _compute_phi_handler(request, context):  # type: ignore[no-untyped-def]
    """Server-side handler for ComputePhi."""
    from scoring_intel.api import abd

    body = {
        "member_id": request.member_id,
        "opportunity_id": request.opportunity_id,
        "goal_mode": request.goal_mode,
        "compliance_config_version": request.compliance_config_version,
    }
    http_result = await abd.compute_phi_for_grpc(body)
    return _pb2.ComputePhiResponse(
        opportunity_id=request.opportunity_id,
        phi=http_result.get("phi", 0.0),
        components=_pb2.FitScoreComponents(
            skill=http_result.get("skill", 0.0),
            seniority=http_result.get("seniority", 0.0),
            geo=http_result.get("geo", 0.0),
            comp=http_result.get("comp", 0.0),
            trigger_recency=http_result.get("trigger_recency", 0.0),
            goal_mode=request.goal_mode,
        ),
        eligible=http_result.get("eligible", False),
        rule_based_fallback=http_result.get("rule_based_fallback", False),
    )


async def _predict_rho_handler(request, context):  # type: ignore[no-untyped-def]
    """Server-side handler for PredictReplyProbability."""
    from scoring_intel.api import rho

    feats = request.features or _pb2.ReplyProbabilityFeatures()
    body = {
        "member_id": feats.member_id,
        "contact_id": feats.contact_id,
        "mutual_count": feats.mutual_count,
        "personalization_score": feats.personalization_score,
        "prior_interaction_flag": feats.prior_interaction_flag,
        "contact_tier": feats.contact_tier,
        "last_interaction_age_days": feats.last_interaction_age_days,
        "tag_match_flags": list(feats.tag_match_flags),
    }
    http_result = await rho.predict_for_grpc(body)
    return _pb2.PredictReplyProbabilityResponse(
        rho=http_result.get("rho", 0.0),
        rule_based_fallback=http_result.get("rule_based_fallback", False),
        labeled_send_count=http_result.get("labeled_send_count", 0),
        model_version=http_result.get("model_version", "rule-based-v1"),
        reason=http_result.get("reason", ""),
    )


async def _calibration_status_handler(request, context):  # type: ignore[no-untyped-def]
    """Server-side handler for CalibrationStatus."""
    from scoring_intel.api import hc

    status = await hc.calibration_status_for_grpc()
    return _pb2.CalibrationStatusResponse(
        h_c_labeled_account_days=status.get("h_c_labeled_account_days", 0),
        rho_labeled_sends=status.get("rho_labeled_sends", 0),
        h_c_calibrated=status.get("h_c_calibrated", False),
        rho_calibrated=status.get("rho_calibrated", False),
        h_c_active_weights_version=status.get("h_c_active_weights_version", ""),
        rho_active_model_version=status.get("rho_active_model_version", ""),
        last_h_c_fit_at_unix_ms=int(status.get("last_h_c_fit_at_unix_ms", 0)),
        last_rho_fit_at_unix_ms=int(status.get("last_rho_fit_at_unix_ms", 0)),
    )


class ScoringIntelServicer(intelligence_pb2_grpc.ScoringIntelServicer):  # type: ignore[misc]
    """Concrete servicers that delegate to the FastAPI handlers.

    Keeping the actual logic in the FastAPI layer means that the HTTP and
    gRPC transports both serve the same code path, and there's only one
    place to fix a bug.
    """

    ComputeH_c = staticmethod(_compute_h_c_handler)
    ComputePhi = staticmethod(_compute_phi_handler)
    PredictReplyProbability = staticmethod(_predict_rho_handler)
    CalibrationStatus = staticmethod(_calibration_status_handler)


async def serve(port: int = 50051, tls_cert_path: Optional[str] = None,
                tls_key_path: Optional[str] = None,
                tls_ca_path: Optional[str] = None) -> None:
    """Start the scoring-intel gRPC server.

    When `tls_cert_path` is provided, the server uses mTLS. The CA bundle
    at `tls_ca_path` is required (client certificates must be signed by
    this CA). When unset, the server runs in plaintext mode for local dev.
    """
    if not GRPC_AVAILABLE or not PROTO_AVAILABLE:
        logger.error(
            "scoring-intel gRPC server cannot start: grpcio=%s proto=%s",
            GRPC_AVAILABLE, PROTO_AVAILABLE,
        )
        return

    server = grpc.aio.server()
    intelligence_pb2_grpc.add_ScoringIntelServicer_to_server(
        ScoringIntelServicer(), server
    )

    bind = f"[::]:{port}"
    if tls_cert_path and tls_key_path:
        with open(tls_cert_path, "rb") as cf, open(tls_key_path, "rb") as kf:
            cert = cf.read()
            key = kf.read()
        ca = None
        if tls_ca_path:
            with open(tls_ca_path, "rb") as caf:
                ca = caf.read()
        creds = grpc.ssl_server_credentials(
            [(key, cert)],
            root_certificates=ca,
            require_client_auth=bool(ca),
        )
        server.add_secure_port(bind, creds)
        logger.info("scoring-intel gRPC server listening on %s (mTLS)", bind)
    else:
        server.add_insecure_port(bind)
        logger.warning(
            "scoring-intel gRPC server listening on %s (PLAINTEXT) — "
            "do not run this in production", bind,
        )

    await server.start()
    await server.wait_for_termination()


def main() -> None:
    port = int(os.environ.get("SCORING_INTEL_GRPC_PORT", "50051"))
    tls_cert = os.environ.get("SCORING_INTEL_TLS_CERT")
    tls_key = os.environ.get("SCORING_INTEL_TLS_KEY")
    tls_ca = os.environ.get("SCORING_INTEL_TLS_CA")
    asyncio.run(serve(port=port, tls_cert_path=tls_cert,
                      tls_key_path=tls_key, tls_ca_path=tls_ca))


if __name__ == "__main__":
    main()
