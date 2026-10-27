#!/usr/bin/env bash
set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$BACKEND"
echo "Seeding dev KB…"
docker compose -f infra/docker/docker-compose.yml exec -T ai-worker \
  python -c "from ai_worker.infra.embedding import embed_query; print('embed ok')" || true
