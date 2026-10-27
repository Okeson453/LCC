#!/usr/bin/env bash
set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$BACKEND"
docker compose -f infra/docker/docker-compose.yml down -v
