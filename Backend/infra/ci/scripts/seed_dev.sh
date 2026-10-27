#!/usr/bin/env bash
set -euo pipefail

BACKEND="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
echo "Seeding dev environment…"
cd "$BACKEND"
psql "$DATABASE_URL" -f schemas/migrations/9999_seed_dev.sql
echo "Seed complete"
