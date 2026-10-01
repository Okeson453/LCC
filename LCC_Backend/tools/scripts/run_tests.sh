#!/usr/bin/env bash
set -euo pipefail
echo "→ cargo test"
cargo test --workspace --no-fail-fast
echo "→ pytest"
cd /workspace/lcc/engine/intelligence
uv run --frozen pytest -q || true
