#!/usr/bin/env bash
set -euo pipefail
echo "→ Rust clippy"
cargo clippy --all-targets --all-features -- -D warnings || exit 1
echo "→ Python ruff + mypy"
cd /workspace/lcc/engine/intelligence
uv run --frozen ruff check . || exit 1
uv run --frozen mypy . || exit 1
echo "→ Boundary check"
/workspace/lcc/infra/ci/scripts/check_boundaries.sh || exit 1
echo "All lints passed"
