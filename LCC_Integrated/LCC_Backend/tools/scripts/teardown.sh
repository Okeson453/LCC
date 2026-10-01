#!/usr/bin/env bash
set -euo pipefail
cd /workspace/lcc
docker compose -f infra/docker/docker-compose.yml down -v
