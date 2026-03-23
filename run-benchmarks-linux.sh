#!/usr/bin/env bash
#
# Linux benchmark runner: applies OS tuning, delegates to run-benchmarks-and-report.sh,
# then reverts tuning.
#
# Must be run as root (or via sudo) for sysctl/cpufreq/IRQ changes.
#
# Prerequisites (require reboot — not handled by this script):
#   Add to kernel boot parameters (e.g. /etc/default/grub GRUB_CMDLINE_LINUX):
#     isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3
#   Then: sudo update-grub && sudo reboot
#
# Usage:
#   sudo ./run-benchmarks-linux.sh [--bench-core 2]
#
set -euo pipefail

BENCH_CORE=2
if [[ "${1:-}" == "--bench-core" ]]; then
    BENCH_CORE="${2:-2}"
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

"$SCRIPT_DIR/run-benchmarks-linux-setup.sh" --bench-core "$BENCH_CORE"

# ── Run benchmarks via shared script ──────────────────────────────────────────
# chrt -f 90: elevate the entire process tree to SCHED_FIFO so it's never preempted..

echo ""
echo "=== Running benchmarks (SCHED_FIFO priority 90, pinning to core $BENCH_CORE) ==="

chrt -f 90 "$SCRIPT_DIR/run-benchmarks-and-report.sh" --pin-core "$BENCH_CORE"

# ── Revert tuning ─────────────────────────────────────────────────────────────

echo ""
echo "=== Reverting runtime tuning ==="
"$SCRIPT_DIR/run-benchmarks-linux-revert.sh"
