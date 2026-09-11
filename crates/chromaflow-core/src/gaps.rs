use crate::types::{Gap, Inventory};

pub fn from_inventory(inv: &Inventory) -> Vec<Gap> {
    let mut gaps = Vec::new();
    let i2c_ok = inv.i2c.iter().any(|n| n.readable);
    if inv.i2c.is_empty() || !i2c_ok {
        gaps.push(Gap {
            id: "i2c_dev_missing".into(),
            detail: "No readable /dev/i2c-* (load i2c-dev + SMBus modules, then log out)".into(),
        });
    }
    let hid_ok = inv.hidraw.iter().any(|n| n.readable);
    if inv.hidraw.is_empty() || !hid_ok {
        gaps.push(Gap {
            id: "udev_hidraw".into(),
            detail: "No readable hidraw nodes (udev/uaccess). Use Install detection support."
                .into(),
        });
    }
    if inv.openrgb.status != "reachable" {
        gaps.push(Gap {
            id: "windows_only_protocol".into(),
            detail: "OpenRGB SDK not reachable on 127.0.0.1:6742. Start openrgb --server locally, or the device has no Linux backend.".into(),
        });
    }
    if inv.hwmon.is_empty() {
        gaps.push(Gap {
            id: "missing_module".into(),
            detail: "No hwmon chips. Install detection support, then rescan.".into(),
        });
    }
    let any_pwm = inv.hwmon.iter().any(|c| !c.pwms.is_empty());
    if !inv.hwmon.is_empty() && !any_pwm {
        gaps.push(Gap {
            id: "no_os_control_channel".into(),
            detail: "Temps exist but no pwm* sysfs nodes — a 12V hub may have no OS control."
                .into(),
        });
    }
    gaps
}
