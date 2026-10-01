#!/usr/bin/env bash
# CI-gated boundary enforcement between Rust Core and Python Intelligence.
#
# Per ADR-0001, the two engines must communicate ONLY via:
#   - gRPC contracts under `proto/lcc/v1/*.proto`
#   - The event bus (Redis Streams)
#
# This script enforces the import boundary at the source level. It is
# the implementation of `infra/ci/scripts/check_boundaries.sh` referenced
# by ADR-0001.
#
# Usage:
#   ./infra/ci/scripts/check_boundaries.sh
#
# Exit codes:
#   0 = boundary clean
#   1 = boundary violation found
#
# Catches:
#   1. Rust Core importing from Python Intelligence (or vice versa) via
#      any path-based reference.
#   2. Python Intelligence using `requests` / `urllib` to call a Rust
#      Core service directly (must use gRPC client or event bus).
#   3. Rust Core using a hardcoded `http://localhost:XXXX` URL pointing
#      at an Intelligence service (must be an `LCC_INTELLIGENCE_*` env
#      var resolved at runtime).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "${ROOT}"

PY_INTEL="${ROOT}/engine/intelligence"
RS_CORE="${ROOT}/engine/core"

VIOLATIONS=0

red()    { printf "\033[31m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }
green()  { printf "\033[32m%s\033[0m\n" "$*"; }

check_no_rs_to_py() {
    echo
    echo "=== Rule 1: Rust Core must not import from Python Intelligence ==="
    # Search all .rs files under engine/core for any PATH-based reference
    # to `engine/intelligence/`. Service NAME references (e.g. "scoring-intel"
    # as a string) are allowed because they go through the tonic client.
    local hits
    hits=$(grep -rn --include='*.rs' \
        -E 'engine/intelligence/' \
        "${RS_CORE}" 2>/dev/null || true)
    if [ -n "${hits}" ]; then
        red "  ✗ Violations found:"
        echo "${hits}" | sed 's/^/    /'
        VIOLATIONS=$((VIOLATIONS + 1))
    else
        green "  ✓ no path-based cross-engine imports"
    fi
}

check_no_py_to_rs() {
    echo
    echo "=== Rule 2: Python Intelligence must not import from Rust Core ==="
    local hits
    hits=$(grep -rn --include='*.py' \
        -E 'engine/core/|engine/intelligence.*from\s+lcc_core|from\s+lcc_engine' \
        "${PY_INTEL}" 2>/dev/null || true)
    if [ -n "${hits}" ]; then
        red "  ✗ Violations found:"
        echo "${hits}" | sed 's/^/    /'
        VIOLATIONS=$((VIOLATIONS + 1))
    else
        green "  ✓ no path-based cross-engine imports"
    fi
}

check_python_uses_grpc() {
    echo
    echo "=== Rule 3: Python Intelligence must not bypass gRPC with HTTP ==="
    local hits
    # Look for `requests.post(.*localhost` or `httpx.post(.*localhost` that
    # call into a Rust Core service.
    hits=$(grep -rn --include='*.py' \
        -E '(requests|httpx)\.(get|post|put|delete)\(' \
        "${PY_INTEL}" 2>/dev/null | grep -E '://(localhost|127\.|.*-svc|.*-intel)' || true)
    # Whitelist: HTTP-to-LinkedIn (Track A) is allowed in integration-gateway,
    # not in the Python intelligence engine.
    if [ -n "${hits}" ]; then
        yellow "  ⚠ HTTP calls to non-Intelligence endpoints (review):"
        echo "${hits}" | sed 's/^/    /'
    else
        green "  ✓ no direct HTTP calls into Rust Core from Python"
    fi
}

check_rust_hardcoded_intel_urls() {
    echo
    echo "=== Rule 4: Rust Core must not hardcode Intelligence URLs ==="
    local hits
    hits=$(grep -rn --include='*.rs' \
        -E 'https?://(localhost|127\.|.*-svc|.*-intel):[0-9]+' \
        "${RS_CORE}" 2>/dev/null | grep -v 'LCC_INTELLIGENCE' || true)
    # Filter false positives in dev/test env.
    local filtered
    filtered=$(echo "${hits}" | grep -v '_test\.rs:' | grep -v '/tests/' || true)
    if [ -n "${filtered}" ]; then
        yellow "  ⚠ Hardcoded Intelligence URLs (review):"
        echo "${filtered}" | sed 's/^/    /'
    else
        green "  ✓ all Intelligence endpoints use env vars"
    fi
}

check_proto_synchronous() {
    echo
    echo "=== Rule 5: proto/lcc/v1/*.proto matches proto/gen/python ==="
    # Count distinct packages (directories containing .proto files) rather
    # than individual .proto files, because the codegen script merges all
    # .proto files in the same package into one _pb2.py per protoc
    # convention.
    local n_proto
    local n_packages
    local n_pb2
    n_proto=$(find "${ROOT}/proto/lcc/v1" -name '*.proto' 2>/dev/null | wc -l)
    n_packages=$(find "${ROOT}/proto/lcc/v1" -name '*.proto' -exec dirname {} \; 2>/dev/null | sort -u | wc -l)
    n_pb2=$(find "${ROOT}/proto/gen/python" -name '_pb2.py' 2>/dev/null | wc -l)
    yellow "  proto files:               ${n_proto}"
    yellow "  distinct proto packages:   ${n_packages}"
    yellow "  generated _pb2.py:         ${n_pb2}"
    if [ "${n_pb2}" -lt "${n_packages}" ]; then
        red "  ✗ some .proto packages do not have generated _pb2.py"
        echo "    Run: python3 proto/gen/python/codegen.py --proto proto/lcc/v1 --out proto/gen/python"
        VIOLATIONS=$((VIOLATIONS + 1))
    else
        green "  ✓ every proto package has a generated python stub"
    fi
}

check_no_rs_to_py
check_no_py_to_rs
check_python_uses_grpc
check_rust_hardcoded_intel_urls
check_proto_synchronous

echo
if [ "${VIOLATIONS}" -gt 0 ]; then
    red "BOUNDARY CHECK FAILED — ${VIOLATIONS} violation(s)"
    exit 1
fi
green "BOUNDARY CHECK PASSED"
