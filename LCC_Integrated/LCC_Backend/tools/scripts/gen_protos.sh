#!/usr/bin/env bash
set -euo pipefail
cd /workspace/lcc
buf generate
echo "Proto generation complete"
