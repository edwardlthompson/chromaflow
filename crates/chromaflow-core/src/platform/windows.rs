//! Windows (AppData + SuperIO/HID) platform backend.

use crate::conflicts;
use crate::cooling::{self, CoolingBackend};
use crate::gaps;
use crate::nvidia_fans;
use crate::probes;
use crate::types::{BinaryProbe, DevNode, Gap, Inventory, OpenRgbProbe};
use std::path::PathBuf;

pub fn cooling_backend() -> Box<dyn CoolingBackend> {
    Box::new(crate::windows::superio::WindowsSuperIoBackend::default())
}

pub fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_CONFIG_HOME") {
        return PathBuf::from(p);
    }
    if let Ok(base) = std::env::var("APPDATA") {
        return PathBuf::from(base).join("chromaflow");
    }
    std::env::temp_dir().join("chromaflow").join("config")
}

pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_DATA_HOME") {
        return PathBuf::from(p);
    }
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(base).join("chromaflow");
    }
    if let Ok(base) = std::env::var("APPDATA") {
        return PathBuf::from(base).join("chromaflow");
    }
    std::env::temp_dir().join("chromaflow")
}

pub fn runtime_dir() -> PathBuf {
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(base).join("chromaflow").join("run");
    }
    std::env::temp_dir().join("chromaflow")
}

pub fn discover_inventory() -> Inventory {
    let snap = cooling::active().snapshot();
    let hidraw = crate::windows::hid::scan_nodes();
    let hid_rgb = crate::windows::hid::scan_hid_rgb(&hidraw);
    let mut inv = Inventory {
        hwmon: snap.chips,
        gpu_fans: nvidia_fans::snapshot(),
        hid_rgb,
        hidraw,
        i2c: Vec::<DevNode>::new(),
        liquidctl: BinaryProbe {
            available: false,
            detail: "liquidctl not used on Windows yet".into(),
        },
        liquidctl_devices: Vec::new(),
        gpu_rgb: Vec::new(),
        openrgb: probes::openrgb(),
        kernel_release: kernel_or_os_release(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::from_inventory(&inv);
    if !inv.hwmon.is_empty() {
        for g in snap.gaps {
            inv.gaps.push(Gap {
                id: "windows_lpc".into(),
                detail: g,
            });
        }
    }
    inv
}

pub fn discover_cooling() -> Inventory {
    let snap = cooling::active().snapshot();
    let mut inv = Inventory {
        hwmon: snap.chips,
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
    if !inv.hwmon.is_empty() {
        for g in snap.gaps {
            inv.gaps.push(Gap {
                id: "windows_lpc".into(),
                detail: g,
            });
        }
    }
    inv
}

pub fn kernel_or_os_release() -> String {
    std::env::var("OS").unwrap_or_else(|_| "windows".into())
}

pub fn euid() -> u32 {
    if std::env::var("CHROMAFLOW_FORCE_ADMIN").ok().as_deref() == Some("1") {
        return 0;
    }
    0xffff_ffff
}

pub fn refuse_if_elevated() -> Result<(), String> {
    if euid() == 0 {
        Err("do not run chromaflow-gui elevated; use the chromaflowd service for LPC".into())
    } else {
        Ok(())
    }
}
