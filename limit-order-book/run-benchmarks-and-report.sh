#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

RESULTS_DIR="bench-results"
mkdir -p "$RESULTS_DIR"

# ── Benchmark reports ─────────────────────────────────────────────────────────

echo "=== Benchmark: LOB v0 (baseline) ==="
mkdir -p "$RESULTS_DIR/v0"
cargo bench --bench lob -- \
    --lob-version v0 \
    --save-json "$RESULTS_DIR/v0/lob.json" \
    --save-md "$RESULTS_DIR/v0/lob.md" \
    --flamegraph

cargo flamegraph --bench lob \
    -o "$RESULTS_DIR/v0/flamegraph.svg" \
    -- --filter Throughput --lob-version "v0"

echo ""
echo "=== Benchmark: LOB v1 (vs v0 baseline) ==="
mkdir -p "$RESULTS_DIR/v1"
cargo bench --bench lob -- \
    --lob-version v1 \
    --baseline "$RESULTS_DIR/v0/lob.json" \
    --save-md "$RESULTS_DIR/v1/lob.md" \
    --flamegraph


cargo flamegraph --bench lob \
    -o "$RESULTS_DIR/v1/flamegraph.svg" \
    -- --filter Throughput --lob-version "v1"

echo ""
echo "Done. Results in ${RESULTS_DIR}/:"
ls  "$RESULTS_DIR/"
