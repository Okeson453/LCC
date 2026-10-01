#!/usr/bin/env bash
# lint_all.sh — top-level lint orchestrator.

set -euo pipefail
echo "→ Running Rust clippy"
cargo clippy --all-targets --all-features -- -D warnings
echo "→ Running Python ruff + mypy"
cd /workspace/lcc/engine/intelligence
uv run --frozen ruff check .
uv run --frozen mypy .
echo "→ Running boundary check"
/workspace/lcc/infra/ci/scripts/check_boundaries.sh
echo "→ Lint complete"
