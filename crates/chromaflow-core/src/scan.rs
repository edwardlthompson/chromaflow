use crate::conflicts;
use crate::gaps;
use crate::hwmon;
use crate::lighting;
use crate::nodes;
use crate::nvidia_fans;
use crate::probes;
use crate::types::{BinaryProbe, Inventory, OpenRgbProbe};

pub fn collect_inventory() -> Inventory {
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
        openrgb: probes::openrgb(),
        kernel_release: kernel_release(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::from_inventory(&inv);
    inv
}

pub fn collect_cooling() -> Inventory {
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
        openrgb: OpenRgbProbe {
            status: "skipped".into(),
            detail: String::new(),
            controllers: Vec::new(),
            sandboxed: false,
            engine_missing: false,
        },
        kernel_release: kernel_release(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::cooling_gaps(&inv);
    inv
}

fn kernel_release() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .trim()
        .to_string()
}
