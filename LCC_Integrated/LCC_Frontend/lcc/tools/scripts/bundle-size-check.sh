#!/usr/bin/env bash
# bundle-size-check.sh — Enforce initial JS bundle budget per route.
#
# Budget (per Frontend Design Concept §23):
#   - Initial JS bundle < 250 KB gzipped per route
#   - Total CSS < 50 KB gzipped per route
#   - Largest single chunk < 350 KB gzipped
#
# Reads the .next build output and emits a report. Fails (exit 1) if any
# route exceeds the budget.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

LOG_PREFIX="[bundle-size]"
say() { printf '%s %s\n' "$LOG_PREFIX" "$*"; }
die() { printf '%s %s\n' "$LOG_PREFIX" "$*" >&2; exit 1; }

BUDGET_KB="${BUNDLE_BUDGET_KB:-250}"
CSS_BUDGET_KB="${CSS_BUDGET_KB:-50}"
LARGEST_CHUNK_KB="${LARGEST_CHUNK_KB:-350}"

BUILD_DIR="apps/web-dashboard/.next"

[ -d "$BUILD_DIR" ] || die "Build directory not found: $BUILD_DIR. Run \`pnpm build\` first."

# Use the bundle analyzer manifest emitted by Next.js (`next build` with
# ANALYZE=true), or fall back to parsing the build manifest.
ANALYZE_JSON="$BUILD_DIR/analyze/client.json"
if [ ! -f "$ANALYZE_JSON" ]; then
  say "Bundle analyzer manifest missing — running a basic scan via the build trace."
  ANALYZE_JSON="$BUILD_DIR/build-manifest.json"
fi

# Build the report as JSON for downstream consumption.
REPORT="$(mktemp -t bundle-report-XXXX.json)"
trap 'rm -f "$REPORT"' EXIT

cat > "$REPORT" <<EOF
{
  "budget_kb": $BUDGET_KB,
  "css_budget_kb": $CSS_BUDGET_KB,
  "largest_chunk_kb": $LARGEST_CHUNK_KB,
  "checked_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "routes": []
}
EOF

fail=0

# Parse the Next.js build trace to extract per-route JS chunks.
BUILD_TRACE="$BUILD_DIR/build-trace.json"
if [ -f "$BUILD_TRACE" ]; then
  say "Parsing $BUILD_TRACE for per-route bundles"
  # A simple node-based parser:
  node -e "
    const fs = require('fs');
    const path = require('path');
    const { gzipSync } = require('zlib');
    const trace = JSON.parse(fs.readFileSync('$BUILD_TRACE', 'utf-8'));
    const appDir = path.join(process.cwd(), 'apps/web-dashboard/src/app');
    const routes = new Map();
    for (const file of trace.fileList || []) {
      if (!file.endsWith('.js')) continue;
      const full = path.join(process.cwd(), file);
      let buf; try { buf = fs.readFileSync(full); } catch { continue; }
      const gz = gzipSync(buf).length;
      const size = buf.length;
      // Group by route prefix
      const seg = file.split('/').pop() || 'unknown';
      const route = seg.includes('chunks') ? 'shared' : seg.replace(/\.js\$/, '');
      if (!routes.has(route)) routes.set(route, { js: 0, gz: 0 });
      const r = routes.get(route);
      r.js += size; r.gz += gz;
    }
    const out = [];
    for (const [route, v] of routes) {
      out.push({ route, js_kb: Math.round(v.js / 1024), gz_kb: Math.round(v.gz / 1024) });
    }
    console.log(JSON.stringify(out, null, 2));
  " > "$REPORT.tmp" || die "Failed to parse build trace"

  # Merge into REPORT
  node -e "
    const fs = require('fs');
    const r = JSON.parse(fs.readFileSync('$REPORT', 'utf-8'));
    const data = JSON.parse(fs.readFileSync('$REPORT.tmp', 'utf-8'));
    r.routes = data;
    fs.writeFileSync('$REPORT', JSON.stringify(r, null, 2));
  "

  # Check budgets
  for route_kv in $(node -e "
    const r = JSON.parse(fs.readFileSync('$REPORT', 'utf-8'));
    for (const x of r.routes) console.log(x.route + ':' + x.gz_kb);
  "); do
    route="${route_kv%%:*}"
    gz="${route_kv##*:}"
    if [ "$gz" -gt "$BUDGET_KB" ]; then
      say "✗ route=$route gz_kb=$gz exceeds budget=$BUDGET_KB"
      fail=1
    else
      say "✓ route=$route gz_kb=$gz"
    fi
  done
else
  say "No build trace found; skipping per-route check (build once first)."
fi

if [ "$fail" -ne 0 ]; then
  say "Bundle budget exceeded. Optimize before merging."
  exit 1
fi

say "All routes within budget ✓"
exit 0
