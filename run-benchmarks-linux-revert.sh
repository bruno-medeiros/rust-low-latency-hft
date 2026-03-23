#!/usr/bin/env bash
#
# Reverts the runtime tuning applied by run-benchmarks-linux.sh.
# Does NOT undo kernel boot parameters (isolcpus, nohz_full, rcu_nocbs) —
# remove those from /etc/default/grub and reboot to fully revert.
#
set -euo pipefail

echo "Restoring CPU governor to 'powersave'..."
for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [ -f "$gov" ] && echo powersave | tee "$gov" > /dev/null
done

if [ -f /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
    echo "Re-enabling Intel turbo boost..."
    echo 0 | tee /sys/devices/system/cpu/intel_pstate/no_turbo > /dev/null
elif [ -f /sys/devices/system/cpu/cpufreq/boost ]; then
    echo "Re-enabling AMD boost..."
    echo 1 | tee /sys/devices/system/cpu/cpufreq/boost > /dev/null
fi

echo "Re-enabling ASLR..."
echo 2 | tee /proc/sys/kernel/randomize_va_space > /dev/null

echo "Re-enabling swap..."
swapon -a 2>/dev/null || true

echo "Runtime tuning reverted."
echo ""
echo "NOTE: Kernel boot parameters (isolcpus, nohz_full, rcu_nocbs) are not"
echo "reverted by this script. Remove them from /etc/default/grub and reboot."
