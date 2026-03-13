#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

RESULTS_DIR="bench-results"
mkdir -p "$RESULTS_DIR"

# ── Benchmark reports ─────────────────────────────────────────────────────────

echo "=== Benchmark: LOB v0 (baseline) ==="
cargo bench --bench lob -- \
    --lob-version v0 \
    --save-json "$RESULTS_DIR/lob-v0.json" \
    --save-md "$RESULTS_DIR/lob-v0.md"
  cargo flamegraph --bench lob \
      -o "$RESULTS_DIR/flamegraph-v0.svg" \
      -- --filter Throughput --lob-version "v0"

echo ""
echo "=== Benchmark: LOB v1 (vs v0 baseline) ==="
cargo bench --bench lob -- \
    --lob-version v1 \
    --baseline "$RESULTS_DIR/lob-v0.json" \
    --save-json "$RESULTS_DIR/lob-v1.json" \
    --save-md "$RESULTS_DIR/lob-v1.md"

cargo flamegraph --bench lob \
    -o "$RESULTS_DIR/flamegraph-v1.svg" \
    -- --filter Throughput --lob-version "v1"

echo ""
echo "Done. Results in ${RESULTS_DIR}/:"
ls -la "$RESULTS_DIR/"
