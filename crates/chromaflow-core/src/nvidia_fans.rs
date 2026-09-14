//! NVIDIA GPU fans via nvidia-settings. Probe only. No NVML.

use crate::types::GpuFan;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

fn display() -> String {
    std::env::var("CHROMAFLOW_DISPLAY")
        .or_else(|_| std::env::var("DISPLAY"))
        .unwrap_or_else(|_| ":0".into())
}

pub fn command() -> Command {
    let dpy = display();
    let mut c = Command::new("nvidia-settings");
    c.env("DISPLAY", &dpy);
    c.arg("-c").arg(&dpy);
    if std::env::var_os("XAUTHORITY").is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let xa = PathBuf::from(home).join(".Xauthority");
            if xa.is_file() {
                c.env("XAUTHORITY", xa);
            }
        }
    }
    c
}

const FAN_TTL: Duration = Duration::from_secs(2);
static GPU_FANS: Mutex<Option<(Instant, Vec<GpuFan>)>> = Mutex::new(None);
static REFRESHING: AtomicBool = AtomicBool::new(false);

pub fn snapshot() -> Vec<GpuFan> {
    let now = Instant::now();
    let stale = GPU_FANS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    if let Some((at, v)) = stale {
        if now.saturating_duration_since(at) >= FAN_TTL {
            kick_refresh();
        }
        return v;
    }
    store(query_fans())
}

fn kick_refresh() {
    if REFRESHING.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = thread::Builder::new().name("nvidia-fans".into()).spawn(|| {
        let fans = query_fans();
        store(fans);
        REFRESHING.store(false, Ordering::SeqCst);
    });
}

fn query_fans() -> Vec<GpuFan> {
    query().map(|t| parse(&t)).unwrap_or_default()
}

fn store(fans: Vec<GpuFan>) -> Vec<GpuFan> {
    if let Ok(mut g) = GPU_FANS.lock() {
        *g = Some((Instant::now(), fans.clone()));
    }
    fans
}

fn query() -> Option<String> {
    let out = command()
        .args(["-q", "GPUCurrentFanSpeedRPM", "-q", "GPUCurrentFanSpeed"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn parse(text: &str) -> Vec<GpuFan> {
    let mut rpm = BTreeMap::<u8, u32>::new();
    let mut pct = BTreeMap::<u8, u8>::new();
    for line in text.lines() {
        let Some(idx) = fan_index(line) else { continue };
        if line.contains("GPUCurrentFanSpeedRPM") {
            if let Some(v) = attr_u32(line) {
                rpm.insert(idx, v);
            }
        } else if line.contains("GPUCurrentFanSpeed") && !line.contains("RPM") {
            if let Some(v) = attr_u32(line) {
                pct.insert(idx, u8::try_from(v.min(100)).unwrap_or(0));
            }
        }
    }
    let mut idxs: Vec<u8> = rpm.keys().chain(pct.keys()).copied().collect();
    idxs.sort_unstable();
    idxs.dedup();
    idxs.into_iter()
        .map(|i| GpuFan {
            id: format!("nvidia:fan:{i}"),
            label: format!("GPU fan {i}"),
            percent: pct.get(&i).copied(),
            rpm: rpm.get(&i).copied(),
            writable: true,
        })
        .collect()
}

fn fan_index(line: &str) -> Option<u8> {
    let rest = line.split("[fan:").nth(1)?;
    rest.split(']').next()?.parse().ok()
}

fn attr_u32(line: &str) -> Option<u32> {
    let rest = line.rsplit(':').next()?.trim();
    rest.trim_end_matches('.').trim().parse().ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_two_nvidia_fans() {
        let text = "Attribute 'GPUCurrentFanSpeedRPM' (host:0[fan:0]): 900.\n\
             Attribute 'GPUCurrentFanSpeed' (host:0[fan:0]): 30.\n\
             Attribute 'GPUCurrentFanSpeedRPM' (host:0[fan:1]): 0.\n\
             Attribute 'GPUCurrentFanSpeed' (host:0[fan:1]): 0.\n";
        let fans = super::parse(text);
        assert_eq!(fans.len(), 2);
        assert_eq!(fans[0].id, "nvidia:fan:0");
        assert_eq!(fans[0].rpm, Some(900));
        assert_eq!(fans[0].percent, Some(30));
        assert_eq!(fans[1].id, "nvidia:fan:1");
        assert_eq!(fans[1].rpm, Some(0));
        let live = "  Attribute 'GPUCurrentFanSpeedRPM' (RYZENmax:0[fan:0]): 0.\n\
              Attribute 'GPUCurrentFanSpeedRPM' (RYZENmax:0[fan:1]): 0.\n\
              Attribute 'GPUCurrentFanSpeed' (RYZENmax:0[fan:0]): 0.\n\
              Attribute 'GPUCurrentFanSpeed' (RYZENmax:0[fan:1]): 0.\n";
        assert_eq!(super::parse(live).len(), 2);
        assert!(super::display().contains(':'));
    }
}
