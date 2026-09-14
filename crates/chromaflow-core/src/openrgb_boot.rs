//! Boot-time OpenRGB settle. One sibling restart if GPU/Keychron missing. No PWM.

use crate::openrgb_spawn::{ensure_sdk, restart_sdk_once};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

fn once() -> bool {
    std::env::var("CHROMAFLOW_DAEMON_ONCE").ok().as_deref() == Some("1")
}

fn wait_x(limit: Duration) {
    let start = Instant::now();
    while start.elapsed() < limit {
        if Path::new("/tmp/.X11-unix/X0").exists() {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

pub fn thin_sdk(names: &str, keychron_hid: bool, nvidia: bool) -> bool {
    let n = names.to_ascii_lowercase();
    if keychron_hid && !(n.contains("keychron") || n.contains("q6")) {
        return true;
    }
    if nvidia && !(n.contains("nvidia") || n.contains("geforce")) {
        return true;
    }
    false
}

fn hid_keychron() -> bool {
    let root = crate::lighting::hidraw_sys_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return false;
    };
    for e in entries.flatten() {
        let text = fs::read_to_string(e.path().join("device/uevent")).unwrap_or_default();
        for line in text.lines() {
            let Some(rest) = line.strip_prefix("HID_ID=") else {
                continue;
            };
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() < 3 {
                continue;
            }
            let hex = parts[1].trim().trim_start_matches('0').to_ascii_lowercase();
            let vid = format!("{:0>4}", if hex.is_empty() { "0" } else { &hex });
            if vid == "3434" {
                return true;
            }
        }
    }
    false
}

fn nvidia_present() -> bool {
    let p = std::env::var("CHROMAFLOW_NVIDIA0").unwrap_or_else(|_| "/dev/nvidia0".into());
    Path::new(&p).exists()
}

fn missing_expected() -> bool {
    let keychron = hid_keychron();
    let nvidia = nvidia_present();
    if !keychron && !nvidia {
        return false;
    }
    let blob = crate::openrgb::probe_fresh()
        .controllers
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    thin_sdk(&blob, keychron, nvidia)
}

pub fn run() -> i32 {
    if !once() {
        wait_x(Duration::from_secs(20));
    }
    ensure_sdk();
    if once() {
        return 0;
    }
    let start = Instant::now();
    let mut did = false;
    loop {
        ensure_sdk();
        if !did && start.elapsed() < Duration::from_secs(45) && missing_expected() {
            did = restart_sdk_once();
            crate::openrgb::drop_probe_cache();
        }
        thread::sleep(Duration::from_secs(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thin_when_hid_or_gpu_missing_from_sdk() {
        assert!(thin_sdk("Aorus RAM", true, false));
        assert!(thin_sdk("Keychron Q6 HE", false, true));
        assert!(!thin_sdk("Q6 HE NVIDIA GeForce", true, true));
        assert!(!thin_sdk("", false, false));
    }
}
