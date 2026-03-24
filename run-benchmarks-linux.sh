#!/usr/bin/env bash
#
# Linux benchmark runner: applies OS tuning, delegates to run-benchmarks-and-report.sh,
# then reverts tuning.
#
# The invoking user must be able to run sudo without a password prompt.
#
# Prerequisites (require reboot — not handled by this script):
#   Add to kernel boot parameters (e.g. /etc/default/grub GRUB_CMDLINE_LINUX):
#     isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3
#   Then: sudo update-grub && sudo reboot
#
# Usage:
#   ./run-benchmarks-linux.sh
#
set -euo pipefail

# Must match isolcpus / nohz_full in kernel boot params (see above).
readonly BENCH_CORE=2
readonly BENCH_CORE_B=3


SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# ── Apply OS tuning (as root) ─────────────────────────────────────────────────

sudo "$SCRIPT_DIR/run-benchmarks-linux-setup.sh"

# ── Run benchmarks as the current user ────────────────────────────────────────

echo ""
echo "=== Running benchmarks ==="

sudo chrt -f 90 sudo -u "$USER" -- "$SCRIPT_DIR/run-benchmarks-and-report.sh" \
    --pin-core "$BENCH_CORE" \
    --pin-core-b "$BENCH_CORE_B"

# ── Revert tuning ─────────────────────────────────────────────────────────────

echo ""
echo "=== Reverting runtime tuning ==="
sudo "$SCRIPT_DIR/run-benchmarks-linux-revert.sh"
