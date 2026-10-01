#!/usr/bin/env bash
set -euo pipefail
cd /workspace/lcc
echo "Seeding dev KB…"
docker compose -f infra/docker/docker-compose.yml exec -T ai-worker \
  python -c "from ai_worker.infra.embedding import embed_query; print('embed ok')" || true
