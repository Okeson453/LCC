#!/usr/bin/env bash
# bootstrap.sh — one-shot local environment bring-up.
set -euo pipefail

cd /workspace/lcc

echo "→ Bringing up infrastructure (postgres, redis, qdrant, observability)"
docker compose -f infra/docker/docker-compose.yml up -d postgres redis qdrant otel-collector prometheus grafana

echo "→ Waiting for postgres to be ready"
for i in {1..30}; do
  if docker compose -f infra/docker/docker-compose.yml exec -T postgres pg_isready -U postgres >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

echo "→ Running migrations"
docker compose -f infra/docker/docker-compose.yml run --rm migrate-runner

echo "→ Seeding dev data"
docker compose -f infra/docker/docker-compose.yml run --rm migrate-runner psql -h postgres -U postgres -d lcc -f /docker-entrypoint-initdb.d/9999_seed_dev.sql || true

echo "→ Done. Bring up the rest with:"
echo "    docker compose -f infra/docker/docker-compose.yml up"
