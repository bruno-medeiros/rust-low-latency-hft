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

# ── Verify prerequisites ─────────────────────────────────────────────────────

echo "=== Checking kernel-level isolation (requires reboot to change) ==="
ISOLATED=$(cat /sys/devices/system/cpu/isolated 2>/dev/null || echo "")
if [[ -z "$ISOLATED" ]]; then
    echo "  WARNING: No isolated CPUs detected."
    echo "  For best results, add 'isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3' to kernel boot params and reboot."
else
    echo "  Isolated CPUs: $ISOLATED"
fi

# ── Apply runtime tuning ─────────────────────────────────────────────────────

echo ""
echo "=== Applying runtime tuning ==="

# Lock CPU frequency to maximum (prevents turbo ramp-up from distorting early samples)
echo "  Setting CPU governor to 'performance'..."
for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [ -f "$gov" ] && echo performance | tee "$gov" > /dev/null
done

# Disable turbo boost for stable clock frequency across all samples
if [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
    echo "  Disabling Intel turbo boost..."
    echo 1 | tee /sys/devices/system/cpu/intel_pstate/no_turbo > /dev/null
elif [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
    echo "  Disabling AMD boost..."
    echo 0 | tee /sys/devices/system/cpu/cpufreq/boost > /dev/null
fi

# Move hardware interrupts off the benchmark core
echo "  Migrating IRQs away from core $BENCH_CORE..."
for irq in /proc/irq/*/smp_affinity_list; do
    echo "0-1" | tee "$irq" > /dev/null 2>&1 || true
done

# Disable ASLR (address randomization adds jitter to pointer-heavy data structures)
echo "  Disabling ASLR..."
echo 0 | tee /proc/sys/kernel/randomize_va_space > /dev/null

# Disable swap (prevents page-out stalls during measurement)
echo "  Disabling swap..."
swapoff -a 2>/dev/null || true

# Drop filesystem caches so each run starts from the same baseline
echo "  Dropping filesystem caches..."
sync && echo 3 | tee /proc/sys/vm/drop_caches > /dev/null

echo "  Runtime tuning applied."

# ── Run benchmarks via shared script ──────────────────────────────────────────
# chrt -f 90: elevate the entire process tree to SCHED_FIFO so it's never preempted..

echo ""
echo "=== Running benchmarks (SCHED_FIFO priority 90, pinning to core $BENCH_CORE) ==="

chrt -f 90 "$SCRIPT_DIR/run-benchmarks-and-report.sh" --pin-core "$BENCH_CORE"

# ── Revert tuning ─────────────────────────────────────────────────────────────

echo ""
echo "=== Reverting runtime tuning ==="
"$SCRIPT_DIR/run-benchmarks-linux-revert.sh"
