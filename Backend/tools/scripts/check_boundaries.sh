#!/usr/bin/env bash
#
# Architectural boundary enforcement.
#
# F-AUDIT-61: `just bounds` called `tools/scripts/check_boundaries.sh`, which
# did not exist. The recipe therefore exited non-zero on every invocation, and
# because it was a "real" command in a justfile nobody had noticed — the
# boundary it was supposed to guard has never actually been checked.
#
# Two boundaries are architectural constraints that no compiler enforces,
# because Cargo happily allows the dependency and the rule lives in a
# `[workspace.metadata]` block. They therefore have to be checked explicitly,
# or a future change can quietly dissolve them.
#
#   1. Integration boundary — only `integration-gateway` may depend on
#      `crates/integrations` (the outbound LinkedIn/third-party client). Every
#      other service must go through the gateway, so that permit-token
#      enforcement and action logging cannot be bypassed by calling the
#      provider client directly. Declared in the root Cargo.toml as
#      `[workspace.metadata.integration-boundary] allowed_importers`.
#
#   2. Engine boundary — the Rust core must not depend on the Python
#      intelligence tier. The intelligence services call *into* the core over
#      gRPC/HTTP; a compile-time dependency would invert that and couple the
#      request path to the ML stack.
#
# Exits 0 when clean, 1 on violation, printing every offender.

set -euo pipefail

cd "$(dirname "$0")/../.."

failures=0

# Read the allow-list from the manifest rather than hardcoding it, so the
# manifest stays the single source of truth for this rule.
allowed_importers=$(python3 - <<'PY'
import re, pathlib
text = pathlib.Path("Cargo.toml").read_text()
m = re.search(r'\[workspace\.metadata\.integration-boundary\](.*?)(\n\[|\Z)',
               text, re.S)
if not m:
    print("integration-gateway")
else:
    a = re.search(r'allowed_importers\s*=\s*\[(.*?)\]', m.group(1), re.S)
    print(",".join(re.findall(r'"([^"]+)"', a.group(1))) if a else "integration-gateway")
PY
)

echo "==> integration boundary (only these may depend on crates/integrations: ${allowed_importers})"

integrations_dep=$(grep -rl --include=Cargo.toml 'path = "crates/integrations"\|path = "\.\./crates/integrations"\|lcc-integrations' . 2>/dev/null || true)

for manifest in $integrations_dep; do
    # The crate's own manifest is not an importer.
    case "$manifest" in
        ./crates/integrations/Cargo.toml) continue ;;
    esac
    ok=0
    for allowed in ${allowed_importers//,/ }; do
        if [[ "$manifest" == *"/${allowed}/"* ]]; then
            ok=1
            break
        fi
    done
    if [ "$ok" -eq 0 ]; then
        echo "::error file=${manifest#./}::depends on crates/integrations; only ${allowed_importers} may. Calls to the provider client must go through the gateway so permit-token enforcement and action logging cannot be bypassed."
        failures=1
    fi
done

echo "==> engine boundary (Rust core must not depend on the Python intelligence tier)"

# A compile-time dependency would show up as a path dependency pointing at the
# intelligence tree, or as a `lcc_intelligence`/`engine.intelligence` import.
if grep -rn --include=Cargo.toml -E 'path *= *"[^"]*engine/intelligence' . 2>/dev/null | grep -v '^./engine/intelligence/'; then
    echo "::error::a Rust manifest declares a path dependency on engine/intelligence; the Rust core must not depend on the Python tier."
    failures=1
fi

if grep -rn --include='*.rs' -E '(^|[^a-zA-Z_])(lcc_intelligence|engine::intelligence)' engine/core crates 2>/dev/null; then
    echo "::error::Rust code under engine/core or crates references the intelligence tier directly."
    failures=1
fi

if [ "$failures" -eq 0 ]; then
    echo "==> boundaries OK"
else
    echo "==> boundary violations found" >&2
fi

exit "$failures"
