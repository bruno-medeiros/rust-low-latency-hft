#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

RESULTS_DIR="$SCRIPT_DIR/bench-results"
mkdir -p "$RESULTS_DIR"

echo "Results dir: $RESULTS_DIR"
if [[ $# -gt 0 ]]; then
    echo "Extra bench args: $*"
fi

# ── Limit order book benchmarks ───────────────────────────────────────────────

echo "=== Benchmark: LOB v0 (baseline) ==="
mkdir -p "$RESULTS_DIR/v0"
cargo bench -p limit-order-book --bench lob -- \
    --lob-version v0 \
    "$@" \
    --save-json "$RESULTS_DIR/v0/lob.json" \
    --save-md "$RESULTS_DIR/v0/lob.md" \
    --flamegraph

cargo flamegraph -p limit-order-book --bench lob \
    -o "$RESULTS_DIR/v0/flamegraph.svg" \
    -- --filter Throughput --lob-version "v0"

echo ""
echo "=== Benchmark: LOB v1 (vs v0 baseline) ==="
mkdir -p "$RESULTS_DIR/v1"
cargo bench -p limit-order-book --bench lob -- \
    --lob-version v1 \
    "$@" \
    --baseline "$RESULTS_DIR/v0/lob.json" \
    --save-md "$RESULTS_DIR/v1/lob.md" \
    --flamegraph

cargo flamegraph -p limit-order-book --bench lob \
    -o "$RESULTS_DIR/v1/flamegraph.svg" \
    -- --filter Throughput --lob-version "v1"

echo ""
echo "=== Benchmark: matching-pipeline LOBSTER (throughput) ==="
mkdir -p "$RESULTS_DIR/matching-pipeline"
cargo bench -p matching-pipeline --bench pipeline -- \
    "$@" \
    --save-json "$RESULTS_DIR/matching-pipeline/pipeline.json" \
    --save-md "$RESULTS_DIR/matching-pipeline/report.md" \
    --flamegraph

cargo flamegraph -p matching-pipeline --bench pipeline \
    -o "$RESULTS_DIR/matching-pipeline/flamegraph.svg" \
    -- --filter Pipeline

echo ""
echo "=== Benchmark: market-data-handler tick-to-trade (latency) ==="
mkdir -p "$RESULTS_DIR/market-data-handler"
cargo bench -p market-data-handler --bench tick_to_trade -- \
    "$@" \
    --save-json "$RESULTS_DIR/market-data-handler/tick_to_trade.json" \
    --save-md "$RESULTS_DIR/market-data-handler/report.md"

rm -f perf.data perf.data.old

echo ""
echo "Done. Results in ${RESULTS_DIR}/:"
ls "$RESULTS_DIR/"
