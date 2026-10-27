#!/usr/bin/env bash
set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
echo "→ Rust clippy"
cargo clippy --all-targets --all-features -- -D warnings || exit 1
echo "→ Python ruff + mypy"
cd "$BACKEND/engine/intelligence"
uv run --frozen ruff check . || exit 1
uv run --frozen mypy . || exit 1
echo "→ Boundary check"
"$BACKEND/infra/ci/scripts/check_boundaries.sh" || exit 1
echo "All lints passed"
