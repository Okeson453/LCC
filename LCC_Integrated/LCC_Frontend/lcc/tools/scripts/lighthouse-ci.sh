#!/usr/bin/env bash
# lighthouse-ci.sh — Run Lighthouse CI against a deployed preview URL.
#
# Required env vars:
#   LHCI_PREVIEW_URL   — URL of the preview deployment
#   LHCI_BUILD_CONTEXT — Build context (PR number, branch name)
#
# Optional env vars:
#   LHCI_TOKEN         — LHCI server upload token
#   LHCI_SERVER_URL    — LHCI server URL (defaults to project default)
#
# Reads the budget file at `tools/scripts/lighthouse-budget.json`.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

LOG_PREFIX="[lighthouse]"
say() { printf '%s %s\n' "$LOG_PREFIX" "$*"; }
die() { printf '%s %s\n' "$LOG_PREFIX" "$*" >&2; exit 1; }

[ -n "${LHCI_PREVIEW_URL:-}" ] || die "LHCI_PREVIEW_URL not set"

BUDGET_FILE="tools/scripts/lighthouse-budget.json"
[ -f "$BUDGET_FILE" ] || die "Budget file not found: $BUDGET_FILE"

# Use @lhci/cli to run an autorun against the preview URL with the budget.
npx --yes @lhci/cli@^0.13.x autorun \
  --collect.url="$LHCI_PREVIEW_URL" \
  --collect.url="$LHCI_PREVIEW_URL/today" \
  --collect.url="$LHCI_PREVIEW_URL/approvals" \
  --collect.url="$LHCI_PREVIEW_URL/content" \
  --collect.url="$LHCI_PREVIEW_URL/analytics" \
  --collect.settings.preset=desktop \
  --collect.settings.skipAudits=uses-http2,uses-long-cache-ttl \
  --assert.assertions.budgets="$BUDGET_FILE" \
  --assert.assertions.categories:performance=90 \
  --assert.assertions.categories:accessibility=95 \
  --assert.assertions.categories:best-practices=90 \
  --assert.assertions.categories:seo=85 \
  --upload.target=lhci \
  --upload.token="${LHCI_TOKEN:-}" \
  --upload.serverUrl="${LHCI_SERVER_URL:-}"

say "Lighthouse CI complete ✓"
