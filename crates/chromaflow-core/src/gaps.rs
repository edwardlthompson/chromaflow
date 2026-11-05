use crate::types::{Gap, Inventory};
use std::path::Path;

pub fn board_name() -> String {
    std::env::var("CHROMAFLOW_BOARD")
        .ok()
        .or_else(|| std::fs::read_to_string("/sys/class/dmi/id/board_name").ok())
        .unwrap_or_default()
}

pub fn ite_primary_missing(board: &str, names: &[&str]) -> bool {
    let board = board.to_ascii_uppercase();
    let dual = board.contains("X570S") || board.contains("X570 AORUS");
    let secondary = names.iter().any(|n| n.starts_with("it879"));
    let primary = names
        .iter()
        .any(|n| n.starts_with("it8688") || n.starts_with("it8689") || n.starts_with("it8686"));
    dual && secondary && !primary
}

pub fn from_inventory(inv: &Inventory) -> Vec<Gap> {
    let mut gaps = Vec::new();
    #[cfg(not(windows))]
    {
        let i2c_ok = inv.i2c.iter().any(|n| n.readable);
        if inv.i2c.is_empty() || !i2c_ok {
            gaps.push(Gap {
                id: "i2c_dev_missing".into(),
                detail: "No readable /dev/i2c-* (load i2c-dev + SMBus modules, then log out)"
                    .into(),
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
    }
    #[cfg(windows)]
    {
        let hid_ok = inv.hidraw.iter().any(|n| n.readable);
        if inv.hidraw.is_empty() || !hid_ok {
            gaps.push(Gap {
                id: "hid_access".into(),
                detail: "No readable HID RGB devices yet. Quit iCUE/Armoury if they lock USB; install the ChromaFlow lighting engine from Support."
                    .into(),
            });
        }
    }
    if inv.openrgb.status != "reachable" {
        gaps.push(Gap {
            id: "windows_only_protocol".into(),
            detail: {
                #[cfg(windows)]
                {
                    "Lighting engine SDK not reachable on 127.0.0.1:6742. Use Support to install/start the private engine (no separate OpenRGB app needed).".into()
                }
                #[cfg(not(windows))]
                {
                    "OpenRGB SDK not reachable on 127.0.0.1:6742. Start openrgb --server locally, or the device has no Linux backend.".into()
                }
            },
        });
    }
    if inv.openrgb.sandboxed {
        gaps.push(Gap {
            id: "openrgb_sandboxed".into(),
            detail: "OpenRGB is running in a sandbox (Flatpak/bwrap) and may not see USB RGB. Use native openrgb --server --server-host 127.0.0.1.".into(),
        });
    }
    if !inv.hid_rgb.is_empty() {
        let sdk = inv
            .openrgb
            .controllers
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let liquid = inv.liquidctl_devices.join(" ");
        let hidden = inv.hid_rgb.iter().any(|h| {
            !crate::lighting::hid_has_linux_backend(
                &h.vendor_id,
                &h.product_id,
                &h.name,
                &sdk,
                &liquid,
            )
        });
        if hidden {
            gaps.push(Gap {
                id: "usb_rgb_not_in_openrgb".into(),
                detail: "USB RGB exists that OpenRGB does not list (often mouse/speakers). Motherboard/keyboard use SDK or VIA. Do not fake sliders.".into(),
            });
        }
    }
    gaps.extend(cooling_gaps(inv));
    gaps
}

pub fn cooling_gaps(inv: &Inventory) -> Vec<Gap> {
    let mut gaps = Vec::new();
    if inv.hwmon.is_empty() {
        #[cfg(windows)]
        {
            let detail = if crate::windows::lpc::bridge_live() {
                "LPC bridge is up but no ITE chips answered (ports read 0xFF). Quit EasyTune/GCC/Fan Control, then Support → Install all and Rescan."
                    .into()
            } else if crate::windows::lpc::superio_bridge_ready() {
                "chromaflow-lpc is installed but the named pipe is down — use Support → Install all to start ChromaFlowLPC."
                    .into()
            } else {
                "Motherboard fans need PawnIO + chromaflow-lpc. Use Support → Install all (no separate OpenRGB app)."
                    .into()
            };
            gaps.push(Gap {
                id: "windows_cooling_setup".into(),
                detail,
            });
        }
        #[cfg(not(windows))]
        {
            gaps.push(Gap {
                id: "missing_module".into(),
                detail: "No hwmon chips. Install detection support, then rescan.".into(),
            });
        }
    }
    let any_pwm = inv.hwmon.iter().any(|c| !c.pwms.is_empty());
    if !inv.hwmon.is_empty() && !any_pwm {
        gaps.push(Gap {
            id: "no_os_control_channel".into(),
            detail: {
                #[cfg(windows)]
                {
                    "Chip listed but no PWM channels — firmware may hide headers or LPC probe is incomplete.".into()
                }
                #[cfg(not(windows))]
                {
                    "Temps exist but no pwm* sysfs nodes — a 12V hub may have no OS control.".into()
                }
            },
        });
    }
    #[cfg(not(windows))]
    {
        let names: Vec<&str> = inv.hwmon.iter().map(|c| c.name.as_str()).collect();
        if ite_primary_missing(&board_name(), &names) {
            gaps.push(Gap {
                id: "ite_primary_missing".into(),
                detail: "Gigabyte dual-ITE board: in-tree it87 bound IT87952E only. Primary IT8689 (CPU_FAN / SYS_FAN) needs out-of-tree it87 DKMS — ChromaFlow does not ship .ko. YAML it8689 is not a real module.".into(),
            });
        }
    }
    if Path::new("/usr/bin/nvidia-settings").is_file() && inv.gpu_fans.is_empty() {
        gaps.push(Gap {
            id: "nvidia_fans_query".into(),
            detail: "nvidia-settings is installed but listed no GPU fans (needs a DISPLAY; hybrid AIO fans stay at 0% idle).".into(),
        });
    }
    gaps
}

#[cfg(test)]
mod tests {
    #[test]
    fn x570s_secondary_only_is_a_gap() {
        assert!(super::ite_primary_missing(
            "X570S AORUS MASTER",
            &["it87952", "k10temp"]
        ));
        assert!(!super::ite_primary_missing(
            "X570S AORUS MASTER",
            &["it87952_800a090a", "it8689_800a090a"]
        ));
        assert!(!super::ite_primary_missing("B650", &["it87952"]));
    }
}
