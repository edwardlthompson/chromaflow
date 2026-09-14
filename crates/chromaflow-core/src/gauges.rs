//! Read-only CPU/GPU/RAM/disk ratios for host LED gauges. No PWM.

use crate::types::HwmonChip;
use serde::Serialize;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const DISK_TTL: Duration = Duration::from_secs(30);
static DISK: Mutex<Option<(Instant, Option<f64>)>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Default)]
pub struct Gauges {
    pub cpu_c: Option<f64>,
    pub gpu_c: Option<f64>,
    pub ram: Option<f64>,
    pub disk: Option<f64>,
    pub ram_c: Option<f64>,
    pub disk_c: Option<f64>,
    pub cpu_load: Option<f64>,
    pub gpu_load: Option<f64>,
    pub note: String,
}

pub fn snapshot(hwmon: &[HwmonChip]) -> Gauges {
    let cpu_c = max_group_c(hwmon, "cpu");
    let gpu = crate::nvidia_smi::cached_gpu();
    let gpu_c = max_group_c(hwmon, "gpu").or(gpu.temp_c);
    let ram = fs::read_to_string("/proc/meminfo")
        .ok()
        .as_deref()
        .and_then(parse_meminfo);
    let disk = disk_used("/");
    let ram_c = max_group_c(hwmon, "ram");
    let disk_c = max_group_c(hwmon, "disk");
    let cpu_load = crate::cpu_load::cached_cpu_load();
    let gpu_load = gpu.load;
    let note = if cpu_c.is_none() && gpu_c.is_none() {
        "CPU/GPU temp not in hwmon".into()
    } else if gpu_c.is_none() {
        "GPU temp unavailable".into()
    } else if cpu_c.is_none() {
        "CPU temp not in hwmon".into()
    } else {
        String::new()
    };
    Gauges {
        cpu_c,
        gpu_c,
        ram,
        disk,
        ram_c,
        disk_c,
        cpu_load,
        gpu_load,
        note,
    }
}

pub fn parse_meminfo(text: &str) -> Option<f64> {
    let mut total = None;
    let mut avail = None;
    let mut free = None;
    let mut buffers = None;
    let mut cached = None;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let Ok(v) = parts.next().unwrap_or("").parse::<f64>() else {
            continue;
        };
        match key {
            "MemTotal:" => total = Some(v),
            "MemAvailable:" => avail = Some(v),
            "MemFree:" => free = Some(v),
            "Buffers:" => buffers = Some(v),
            "Cached:" => cached = Some(v),
            _ => {}
        }
    }
    let total = total.filter(|t| *t > 0.0)?;
    let used = match avail {
        Some(a) => total - a,
        None => total - free.unwrap_or(0.0) - buffers.unwrap_or(0.0) - cached.unwrap_or(0.0),
    };
    Some((used / total).clamp(0.0, 1.0))
}

pub fn parse_df_p(text: &str) -> Option<f64> {
    let line = text.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 6 {
        return None;
    }
    let blocks: f64 = cols[1].parse().ok()?;
    let used: f64 = cols[2].parse().ok()?;
    if blocks <= 0.0 {
        return None;
    }
    Some((used / blocks).clamp(0.0, 1.0))
}

pub fn disk_used(path: &str) -> Option<f64> {
    let now = Instant::now();
    if let Ok(g) = DISK.lock() {
        if let Some((at, v)) = g.as_ref() {
            if now.saturating_duration_since(*at) < DISK_TTL {
                return *v;
            }
        }
    }
    let out = Command::new("df").args(["-P", path]).output().ok();
    let v = out.and_then(|o| {
        if !o.status.success() {
            return None;
        }
        parse_df_p(&String::from_utf8_lossy(&o.stdout))
    });
    if let Ok(mut g) = DISK.lock() {
        *g = Some((now, v));
    }
    v
}

fn max_group_c(hwmon: &[HwmonChip], want: &str) -> Option<f64> {
    let mut best: Option<f64> = None;
    for chip in hwmon {
        for row in &chip.temps {
            if channel_group(&chip.name, &row.label) != want {
                continue;
            }
            let Some(c) = milli_c(&row.value) else {
                continue;
            };
            if !(-40.0..=125.0).contains(&c) {
                continue;
            }
            best = Some(best.map(|b| b.max(c)).unwrap_or(c));
        }
    }
    best
}

fn channel_group(chip: &str, label: &str) -> &'static str {
    let n = format!("{chip} {label}").to_ascii_lowercase();
    if n.contains("jc42")
        || n.contains("spd5118")
        || n.contains("dimm")
        || n.contains("sodimm")
        || n.contains("ddr3")
        || n.contains("ddr4")
        || n.contains("ddr5")
    {
        "ram"
    } else if n.contains("nvme") || n.contains("drivetemp") {
        "disk"
    } else if n.contains("amdgpu") || n.contains("nvidia") || n.contains("nouveau") {
        "gpu"
    } else if n.contains("k10") || n.contains("coretemp") || n.contains("zenpower") {
        "cpu"
    } else {
        "other"
    }
}

fn milli_c(raw: &str) -> Option<f64> {
    let n: f64 = raw.trim().parse().ok()?;
    Some(n / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HwmonChip, LabeledValue};

    #[test]
    fn meminfo_available_and_fallback() {
        let a = parse_meminfo("MemTotal: 1000 kB\nMemAvailable: 250 kB\n").unwrap();
        assert!((a - 0.75).abs() < 0.001);
        let b =
            parse_meminfo("MemTotal: 1000 kB\nMemFree: 100 kB\nBuffers: 50 kB\nCached: 50 kB\n")
                .unwrap();
        assert!((b - 0.8).abs() < 0.001);
        assert!(parse_meminfo("").is_none());
        assert!(parse_meminfo("MemTotal: 0 kB\nMemAvailable: 0 kB\n").is_none());
    }

    #[test]
    fn df_and_temp_max() {
        let df = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda1 100 40 60 40% /\n";
        assert!((parse_df_p(df).unwrap() - 0.4).abs() < 0.001);
        let chips = vec![
            HwmonChip {
                name: "k10temp".into(),
                path: "/sys/class/hwmon/hwmon0".into(),
                temps: vec![
                    LabeledValue {
                        label: "temp1_input".into(),
                        value: "40000".into(),
                    },
                    LabeledValue {
                        label: "temp2_input".into(),
                        value: "70000".into(),
                    },
                ],
                fans: vec![],
                pwms: vec![],
            },
            HwmonChip {
                name: "amdgpu".into(),
                path: "/sys/class/hwmon/hwmon1".into(),
                temps: vec![LabeledValue {
                    label: "temp1_input".into(),
                    value: "50000".into(),
                }],
                fans: vec![],
                pwms: vec![],
            },
            HwmonChip {
                name: "nvme".into(),
                path: "/sys/class/hwmon/hwmon2".into(),
                temps: vec![LabeledValue {
                    label: "temp1_input".into(),
                    value: "56850".into(),
                }],
                fans: vec![],
                pwms: vec![],
            },
            HwmonChip {
                name: "jc42".into(),
                path: "/sys/class/hwmon/hwmon3".into(),
                temps: vec![LabeledValue {
                    label: "temp1_input".into(),
                    value: "42000".into(),
                }],
                fans: vec![],
                pwms: vec![],
            },
        ];
        let g = snapshot(&chips);
        assert!((g.cpu_c.unwrap() - 70.0).abs() < 0.01);
        assert!((g.gpu_c.unwrap() - 50.0).abs() < 0.01);
        assert!((g.disk_c.unwrap() - 56.85).abs() < 0.01);
        assert!((g.ram_c.unwrap() - 42.0).abs() < 0.01);
        let none = snapshot(&[]);
        assert!(none.cpu_c.is_none());
        assert!(none.gpu_c.is_some() || none.note.contains("hwmon"));
    }

    #[test]
    fn ram_from_dimm_label_not_ite_thermistor() {
        let chips = vec![
            HwmonChip {
                name: "it8688".into(),
                path: "/sys/class/hwmon/hwmon0".into(),
                temps: vec![
                    LabeledValue {
                        label: "System".into(),
                        value: "80000".into(),
                    },
                    LabeledValue {
                        label: "DIMM".into(),
                        value: "41000".into(),
                    },
                ],
                fans: vec![],
                pwms: vec![],
            },
            HwmonChip {
                name: "it87952".into(),
                path: "/sys/class/hwmon/hwmon1".into(),
                temps: vec![LabeledValue {
                    label: "temp1_input".into(),
                    value: "51000".into(),
                }],
                fans: vec![],
                pwms: vec![],
            },
        ];
        let g = snapshot(&chips);
        assert!((g.ram_c.unwrap() - 41.0).abs() < 0.01);
        assert!(g.cpu_c.is_none());
    }
}
