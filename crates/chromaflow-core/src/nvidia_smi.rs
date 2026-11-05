//! Cached `nvidia-smi` GPU °C and util. Gauges only, never per LED frame. No NVML.

use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::win_cmd;

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

fn query_gpu_text() -> Option<String> {
    #[cfg(windows)]
    {
        // Never use Windows `timeout.exe` — it allocates a console and steals focus.
        let mut cmd = Command::new("nvidia-smi");
        cmd.args([
            "--query-gpu=temperature.gpu,utilization.gpu",
            "--format=csv,noheader,nounits",
        ]);
        let out = win_cmd::hidden_capture(cmd).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }
    #[cfg(not(windows))]
    {
        let out = Command::new("timeout")
            .args([
                "1",
                "nvidia-smi",
                "--query-gpu=temperature.gpu,utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }
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
    let v = query_gpu_text()
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

    #[cfg(windows)]
    #[test]
    fn windows_query_does_not_use_timeout_exe() {
        // Regression: Command::new("timeout") on Windows is timeout.exe (focus steal).
        let src = include_str!("nvidia_smi.rs");
        let win_block = src
            .split("#[cfg(windows)]")
            .nth(1)
            .unwrap_or("")
            .split("#[cfg(not(windows))]")
            .next()
            .unwrap_or("");
        assert!(
            !win_block.contains("Command::new(\"timeout\")"),
            "Windows nvidia-smi path must not invoke timeout.exe"
        );
        assert!(win_block.contains("nvidia-smi"));
    }
}
