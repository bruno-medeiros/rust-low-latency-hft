#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

for version in v1 v0; do
    echo "=== Flamegraph: LOB ${version} — Throughput ==="
    cargo flamegraph --bench lob \
        -o "flamegraph-${version}.svg" \
        -- --filter Throughput --lob-version "$version"
done

echo ""
echo "Done. Flamegraphs:"
echo "  ${SCRIPT_DIR}/flamegraph-v0.svg"
echo "  ${SCRIPT_DIR}/flamegraph-v1.svg"