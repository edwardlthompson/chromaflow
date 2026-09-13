//! Cached `nvidia-smi` GPU °C and util. Gauges only, never per LED frame. No NVML.

use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const GPU_TTL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, Default)]
pub struct GpuSmi {
    pub temp_c: Option<f64>,
    pub load: Option<f64>,
}

static GPU_SMI: Mutex<Option<(Instant, GpuSmi)>> = Mutex::new(None);

pub fn parse_nvidia_smi(text: &str) -> Option<f64> {
    let mut best: Option<f64> = None;
    for line in text.lines() {
        let raw = line.split(',').next().unwrap_or("").trim();
        let Ok(v) = raw.parse::<f64>() else { continue };
        if !(-40.0..=125.0).contains(&v) {
            continue;
        }
        best = Some(best.map(|b| b.max(v)).unwrap_or(v));
    }
    best
}

pub fn parse_nvidia_util(text: &str) -> Option<f64> {
    let mut best: Option<f64> = None;
    for line in text.lines() {
        let raw = line.split(',').nth(1).unwrap_or("").trim();
        let Ok(v) = raw.parse::<f64>() else { continue };
        if !(0.0..=100.0).contains(&v) {
            continue;
        }
        let r = v / 100.0;
        best = Some(best.map(|b| b.max(r)).unwrap_or(r));
    }
    best
}

pub fn cached_gpu() -> GpuSmi {
    let now = Instant::now();
    if let Ok(g) = GPU_SMI.lock() {
        if let Some((at, v)) = g.as_ref() {
            if now.saturating_duration_since(*at) < GPU_TTL {
                return *v;
            }
        }
    }
    let out = Command::new("nvidia-smi")
        .args([
            "--query-gpu=temperature.gpu,utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok();
    let v = out
        .and_then(|o| {
            if !o.status.success() {
                return None;
            }
            Some(String::from_utf8_lossy(&o.stdout).into_owned())
        })
        .map(|text| GpuSmi {
            temp_c: parse_nvidia_smi(&text),
            load: parse_nvidia_util(&text),
        })
        .unwrap_or_default();
    if let Ok(mut g) = GPU_SMI.lock() {
        *g = Some((now, v));
    }
    v
}

pub fn cached_gpu_c() -> Option<f64> {
    cached_gpu().temp_c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_nounits() {
        assert_eq!(parse_nvidia_smi("46\n").unwrap(), 46.0);
        assert_eq!(parse_nvidia_smi("40\n52\n").unwrap(), 52.0);
        assert_eq!(parse_nvidia_smi("46, N/A\n").unwrap(), 46.0);
        assert_eq!(parse_nvidia_util("46, 12\n").unwrap(), 0.12);
        assert_eq!(parse_nvidia_util("40, 8\n52, 90\n").unwrap(), 0.9);
        assert!(parse_nvidia_smi("N/A\n").is_none());
        assert!(parse_nvidia_util("46, N/A\n").is_none());
        assert!(parse_nvidia_smi("").is_none());
    }
}
