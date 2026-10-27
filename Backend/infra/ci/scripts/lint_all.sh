#!/usr/bin/env bash
# lint_all.sh — top-level lint orchestrator.

set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
echo "→ Running Rust clippy"
cargo clippy --all-targets --all-features -- -D warnings
echo "→ Running Python ruff + mypy"
cd "$BACKEND/engine/intelligence"
uv run --frozen ruff check .
uv run --frozen mypy .
echo "→ Running boundary check"
"$BACKEND/infra/ci/scripts/check_boundaries.sh"
echo "→ Lint complete"
