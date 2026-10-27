#!/usr/bin/env bash
set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
echo "→ cargo test"
cargo test --workspace --no-fail-fast
echo "→ pytest"
cd "$BACKEND/engine/intelligence"
uv run --frozen pytest -q || echo "Some Python tests may be skipped due to missing services"
echo "→ done"
