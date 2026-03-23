#!/usr/bin/env bash
#
# Linux benchmark runtime tuning: 
# Must be run as root (or via sudo) for sysfs/cpufreq/IRQ changes.
#
# Prerequisites (require reboot — not handled by this script):
#   Add to kernel boot parameters (e.g. /etc/default/grub GRUB_CMDLINE_LINUX):
#     isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3
#   Then: sudo update-grub && sudo reboot
#
# Usage:
#   sudo ./run-benchmarks-linux-setup.sh [--bench-core 2]
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

echo "  Setting CPU governor to 'performance'..."
for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [ -f "$gov" ] && echo performance | tee "$gov" > /dev/null
done

if [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
    echo "  Disabling Intel turbo boost..."
    echo 1 | tee /sys/devices/system/cpu/intel_pstate/no_turbo > /dev/null
elif [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
    echo "  Disabling AMD boost..."
    echo 0 | tee /sys/devices/system/cpu/cpufreq/boost > /dev/null
fi

echo "  Migrating IRQs away from core $BENCH_CORE..."
for irq in /proc/irq/*/smp_affinity_list; do
    echo "0-1" | tee "$irq" > /dev/null 2>&1 || true
done

echo "  Disabling ASLR..."
echo 0 | tee /proc/sys/kernel/randomize_va_space > /dev/null

echo "  Disabling swap..."
swapoff -a 2>/dev/null || true

echo "  Runtime tuning applied."
