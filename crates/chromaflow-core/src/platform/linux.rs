//! Linux (XDG + hwmon) platform backend.

use crate::conflicts;
use crate::cooling::{CoolingBackend, LinuxHwmonBackend};
use crate::gaps;
use crate::hwmon;
use crate::lighting;
use crate::nodes;
use crate::nvidia_fans;
use crate::probes;
use crate::types::{BinaryProbe, Inventory, OpenRgbProbe};
use std::path::PathBuf;

pub fn cooling_backend() -> Box<dyn CoolingBackend> {
    Box::new(LinuxHwmonBackend)
}

pub fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_CONFIG_HOME") {
        return PathBuf::from(p);
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
        });
    base.join("chromaflow")
}

pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_DATA_HOME") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(p).join("chromaflow");
    }
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/share/chromaflow"))
        .unwrap_or_else(|_| std::env::temp_dir().join("chromaflow"))
}

pub fn runtime_dir() -> PathBuf {
    if let Ok(p) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(p).join("chromaflow");
    }
    std::env::temp_dir().join("chromaflow")
}

pub fn discover_inventory() -> Inventory {
    let hwmon_root = hwmon::hwmon_root();
    let dev_root = nodes::dev_root();
    let hidraw = nodes::scan_prefix(&dev_root, "hidraw");
    let mut inv = Inventory {
        hwmon: hwmon::scan(&hwmon_root),
        gpu_fans: nvidia_fans::snapshot(),
        hid_rgb: lighting::scan(&lighting::hidraw_sys_root(), &hidraw),
        hidraw,
        i2c: nodes::scan_prefix(&dev_root, "i2c-"),
        liquidctl: probes::liquidctl(),
        liquidctl_devices: lighting::liquidctl_devices(),
        gpu_rgb: crate::gpu_apply::devices(),
        openrgb: probes::openrgb(),
        kernel_release: kernel_or_os_release(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::from_inventory(&inv);
    inv
}

pub fn discover_cooling() -> Inventory {
    let hwmon_root = hwmon::hwmon_root();
    let mut inv = Inventory {
        hwmon: hwmon::scan(&hwmon_root),
        gpu_fans: nvidia_fans::snapshot(),
        hid_rgb: Vec::new(),
        hidraw: Vec::new(),
        i2c: Vec::new(),
        liquidctl: BinaryProbe {
            available: false,
            detail: String::new(),
        },
        liquidctl_devices: Vec::new(),
        gpu_rgb: Vec::new(),
        openrgb: OpenRgbProbe {
            status: "skipped".into(),
            detail: String::new(),
            controllers: Vec::new(),
            sandboxed: false,
            engine_missing: false,
        },
        kernel_release: kernel_or_os_release(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::cooling_gaps(&inv);
    inv
}

pub fn kernel_or_os_release() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn euid() -> u32 {
    let Ok(text) = std::fs::read_to_string("/proc/self/status") else {
        return 0xffff_ffff;
    };
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("Uid:") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let _real = parts.next();
        if let Some(effective) = parts.next() {
            if let Ok(id) = effective.parse() {
                return id;
            }
        }
    }
    0xffff_ffff
}

pub fn refuse_if_elevated() -> Result<(), String> {
    if euid() == 0 {
        Err("do not run as root; use Install detection support (polkit)".into())
    } else {
        Ok(())
    }
}
