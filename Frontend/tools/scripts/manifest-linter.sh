#!/usr/bin/env bash
# Lint a Chromium MV3 manifest. Checks required fields and hostname pinning.
set -euo pipefail
file="${1:?manifest path}"
test -f "$file" || { echo "manifest not found"; exit 1; }
require() {
  local jq_pat="$1"; local label="$2"
  v=$(grep -E "\"${jq_pat}\"" "$file" || true)
  if [ -z "$v" ]; then echo "missing: $label"; exit 1; fi
}
require 'manifest_version' 'manifest_version'
require 'name' 'name'
require 'version' 'version'
grep -q '"minimum_chrome_version": "120"' "$file" || { echo "must set minimum_chrome_version to 120+"; exit 1; }
grep -q 'https://www.linkedin.com' "$file" || { echo "host_permissions must include LinkedIn"; exit 1; }
echo "manifest ok: $file"
