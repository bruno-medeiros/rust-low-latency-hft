//! Best-effort snapshot of OS settings that mirror `run-benchmarks-linux-setup.sh` tuning targets.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeTuningInfo {
    /// Kernel `isolcpus` / `/sys/devices/system/cpu/isolated` (Linux only).
    pub isolated_cpus: String,
    /// CPU frequency governor from sysfs (Linux); power hints are not exposed the same way on macOS.
    pub cpu_frequency_governor: String,
    /// Turbo / boost policy where readable from sysfs (Linux).
    pub turbo_boost: String,
    /// IRQ `smp_affinity_list` sample (Linux); N/A on macOS.
    pub irq_affinity: String,
    /// ASLR / VA randomization (`randomize_va_space` or `kern.randomize_va_space`).
    pub aslr: String,
    /// Swap devices or usage summary.
    pub swap: String,
}

impl RuntimeTuningInfo {
    pub fn detect() -> Self {
        detect_impl()
    }
}

/// Inserts runtime-tuning key/value rows (same labels as the benchmark report metadata table).
pub fn append_runtime_tuning_params(
    params: &mut BTreeMap<String, String>,
    tuning: &RuntimeTuningInfo,
) {
    params.insert("Isolated CPUs".into(), tuning.isolated_cpus.clone());
    params.insert("CPU governor".into(), tuning.cpu_frequency_governor.clone());
    params.insert("Turbo / boost".into(), tuning.turbo_boost.clone());
    params.insert("IRQ affinity (sample)".into(), tuning.irq_affinity.clone());
    params.insert("ASLR".into(), tuning.aslr.clone());
    params.insert("Swap".into(), tuning.swap.clone());
}

#[cfg(target_os = "linux")]
fn detect_impl() -> RuntimeTuningInfo {
    RuntimeTuningInfo {
        isolated_cpus: detect_isolated_cpus_linux(),
        cpu_frequency_governor: detect_governor_linux(),
        turbo_boost: detect_turbo_linux(),
        irq_affinity: detect_irq_affinity_linux(),
        aslr: detect_aslr_linux(),
        swap: detect_swap_linux(),
    }
}

#[cfg(target_os = "macos")]
fn detect_impl() -> RuntimeTuningInfo {
    RuntimeTuningInfo {
        isolated_cpus: "not applicable (macOS; no isolcpus sysfs — use thread affinity / QoS)"
            .into(),
        cpu_frequency_governor: "not exposed via sysfs (macOS; see `pmset -g` / Energy settings)"
            .into(),
        turbo_boost: "not exposed via sysfs (macOS)".into(),
        irq_affinity: "not applicable (macOS)".into(),
        aslr: detect_aslr_macos(),
        swap: detect_swap_macos(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detect_impl() -> RuntimeTuningInfo {
    RuntimeTuningInfo {
        isolated_cpus: "unknown (platform not Linux/macOS)".into(),
        cpu_frequency_governor: "unknown (platform not Linux/macOS)".into(),
        turbo_boost: "unknown (platform not Linux/macOS)".into(),
        irq_affinity: "unknown (platform not Linux/macOS)".into(),
        aslr: "unknown (platform not Linux/macOS)".into(),
        swap: "unknown (platform not Linux/macOS)".into(),
    }
}

#[cfg(target_os = "linux")]
fn detect_isolated_cpus_linux() -> String {
    match std::fs::read_to_string("/sys/devices/system/cpu/isolated") {
        Ok(s) => {
            let t = s.trim();
            if t.is_empty() {
                "none listed (empty /sys/.../isolated; add isolcpus=… boot param for isolation)"
                    .into()
            } else {
                t.to_string()
            }
        }
        Err(e) => format!("unavailable ({e})"),
    }
}

#[cfg(target_os = "linux")]
fn detect_governor_linux() -> String {
    use std::collections::BTreeMap;

    let Ok(rd) = std::fs::read_dir("/sys/devices/system/cpu") else {
        return "unavailable (cannot read /sys/devices/system/cpu)".into();
    };

    let mut paths: Vec<_> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let suffix = name.strip_prefix("cpu")?;
            if suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty() {
                Some(e.path().join("cpufreq/scaling_governor"))
            } else {
                None
            }
        })
        .collect();
    paths.sort();

    let mut gov = Vec::new();
    for p in paths {
        if let Ok(s) = std::fs::read_to_string(&p) {
            gov.push(s.trim().to_string());
        }
    }

    if gov.is_empty() {
        return "unavailable (no cpufreq scaling_governor sysfs entries)".into();
    }
    if gov.iter().all(|g| g == &gov[0]) {
        format!("{} (all {} CPUs)", gov[0], gov.len())
    } else {
        let mut counts = BTreeMap::new();
        for g in &gov {
            *counts.entry(g.clone()).or_insert(0) += 1;
        }
        let summary: Vec<String> = counts.iter().map(|(k, v)| format!("{k}:{v}")).collect();
        format!("mixed ({})", summary.join(", "))
    }
}

#[cfg(target_os = "linux")]
fn detect_turbo_linux() -> String {
    if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        return match s.trim() {
            "1" => "disabled (intel_pstate no_turbo=1)".into(),
            "0" => "enabled (intel_pstate no_turbo=0)".into(),
            other => format!("intel_pstate no_turbo={other}"),
        };
    }
    if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost") {
        return match s.trim() {
            "0" => "disabled (AMD cpufreq boost=0)".into(),
            "1" => "enabled (AMD cpufreq boost=1)".into(),
            other => format!("AMD cpufreq boost={other}"),
        };
    }
    "unavailable (no intel_pstate/no_turbo or cpufreq/boost sysfs)".into()
}

#[cfg(target_os = "linux")]
fn detect_irq_affinity_linux() -> String {
    const SAMPLE_CAP: usize = 64;

    let Ok(entries) = std::fs::read_dir("/proc/irq") else {
        return "unavailable (cannot read /proc/irq)".into();
    };

    let mut affinities = Vec::new();
    for e in entries.flatten() {
        let p = e.path().join("smp_affinity_list");
        if p.is_file()
            && let Ok(s) = std::fs::read_to_string(&p)
        {
            let t = s.trim().to_string();
            if !t.is_empty() {
                affinities.push(t);
                if affinities.len() >= SAMPLE_CAP {
                    break;
                }
            }
        }
    }

    if affinities.is_empty() {
        return "no readable smp_affinity_list (permissions or kernel)".into();
    }
    let first = &affinities[0];
    if affinities.iter().all(|a| a == first) {
        format!("{first} (uniform across {} sampled IRQs)", affinities.len())
    } else {
        format!("mixed ({} sampled IRQs; first={})", affinities.len(), first)
    }
}

#[cfg(target_os = "linux")]
fn detect_aslr_linux() -> String {
    match std::fs::read_to_string("/proc/sys/kernel/randomize_va_space") {
        Ok(s) => match s.trim() {
            "0" => "disabled (randomize_va_space=0)".into(),
            "1" => "enabled partial (randomize_va_space=1)".into(),
            "2" => "enabled full (randomize_va_space=2)".into(),
            other => format!("randomize_va_space={other}"),
        },
        Err(e) => format!("unavailable ({e})"),
    }
}

#[cfg(target_os = "linux")]
fn detect_swap_linux() -> String {
    match std::fs::read_to_string("/proc/swaps") {
        Ok(s) => {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() <= 1 {
                "none active (/proc/swaps header only)".into()
            } else {
                let mut parts = Vec::new();
                for line in lines.iter().skip(1) {
                    let mut it = line.split_whitespace();
                    if let (Some(dev), Some(t), Some(used)) = (it.next(), it.next(), it.next()) {
                        parts.push(format!("{dev} type={t} used={used}"));
                    }
                }
                if parts.is_empty() {
                    s.trim().to_string()
                } else {
                    parts.join("; ")
                }
            }
        }
        Err(e) => format!("unavailable ({e})"),
    }
}

#[cfg(target_os = "macos")]
fn detect_aslr_macos() -> String {
    use std::process::Command;

    let out = Command::new("sysctl")
        .args(["-n", "kern.randomize_va_space"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match t.as_str() {
                "0" => "disabled (kern.randomize_va_space=0)".into(),
                "1" => "enabled partial (kern.randomize_va_space=1)".into(),
                "2" => "enabled full (kern.randomize_va_space=2)".into(),
                _ => format!("kern.randomize_va_space={t}"),
            }
        }
        Ok(o) => format!(
            "sysctl failed (status {}): {}",
            o.status,
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => format!("unavailable ({e})"),
    }
}

#[cfg(target_os = "macos")]
fn detect_swap_macos() -> String {
    use std::process::Command;

    let out = Command::new("sysctl").args(["-n", "vm.swapusage"]).output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Ok(o) => format!(
            "sysctl failed (status {}): {}",
            o.status,
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => format!("unavailable ({e})"),
    }
}
