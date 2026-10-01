#!/usr/bin/env bash
set -euo pipefail
echo "Seeding dev environment…"
cd /workspace/lcc
psql "$DATABASE_URL" -f schemas/migrations/9999_seed_dev.sql
echo "Seed complete"
