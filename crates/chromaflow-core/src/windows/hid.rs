//! Windows HID enumeration for native lighting ports + OpenRGB discovery hints.

use crate::types::{DevNode, HidRgb};
use std::process::Command;

/// Enumerate USB HID devices. Fixture mode supports CI without real hardware.
pub fn scan_nodes() -> Vec<DevNode> {
    if let Ok(raw) = std::env::var("CHROMAFLOW_HID_FIXTURE") {
        if raw == "1" {
            return vec![DevNode {
                path: r"\\.\HID#VID_3434".into(),
                readable: true,
            }];
        }
    }
    pnp_hid_nodes()
}

pub fn scan_hid_rgb(nodes: &[DevNode]) -> Vec<HidRgb> {
    nodes
        .iter()
        .filter(|n| n.readable)
        .filter_map(|n| parse_hid_path(&n.path))
        .collect()
}

fn parse_hid_path(path: &str) -> Option<HidRgb> {
    let upper = path.to_ascii_uppercase();
    let vid = capture_hex(&upper, "VID_")?;
    let pid = capture_hex(&upper, "PID_").unwrap_or_else(|| "0000".into());
    let name = friendly_name(&vid, &pid, path);
    Some(HidRgb {
        path: path.to_string(),
        name: name.clone(),
        kind: "hid".into(),
        vendor_id: vid,
        product_id: pid,
        readable: true,
        hid_name: name,
        manufacturer: String::new(),
        product: String::new(),
    })
}

fn capture_hex(hay: &str, key: &str) -> Option<String> {
    let i = hay.find(key)?;
    let rest = &hay[i + key.len()..];
    let hex: String = rest
        .chars()
        .take(4)
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    if hex.len() == 4 {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

fn friendly_name(vid: &str, pid: &str, path: &str) -> String {
    match (vid, pid) {
        ("3434", _) => "Keychron keyboard".into(),
        ("1038", "1a00") => "SteelSeries Arena 7".into(),
        ("1038", "1856") => "SteelSeries Prime Neo".into(),
        ("1038", _) => format!("SteelSeries HID {pid}"),
        ("048d", "5702") => "Gigabyte RGB Fusion".into(),
        _ => path
            .rsplit('\\')
            .next()
            .unwrap_or("Windows HID")
            .to_string(),
    }
}

/// Present USB HID with lighting-relevant VIDs (Keychron, SteelSeries, ITE Fusion).
fn pnp_hid_nodes() -> Vec<DevNode> {
    let script = "Get-PnpDevice -PresentOnly -ErrorAction SilentlyContinue | \
        Where-Object { \
          $_.InstanceId -match 'VID_3434|VID_1038|VID_048D' -or \
          $_.HardwareID -match 'VID_3434|VID_1038|VID_048D' \
        } | ForEach-Object { \
          $id = $_.InstanceId; \
          if (-not $id) { $id = ($_.HardwareID | Select-Object -First 1) }; \
          if ($id) { $id } \
        } | Select-Object -Unique";
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    let Ok(out) = crate::win_cmd::hidden_capture(cmd).output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| {
            let u = l.to_ascii_uppercase();
            u.contains("VID_3434") || u.contains("VID_1038") || u.contains("VID_048D")
        })
        .map(|l| DevNode {
            path: l.to_string(),
            readable: true,
        })
        .collect()
}

/// Quit SteelSeries GG / Engine so HID + OpenRGB can claim Arena / mice / headset.
/// Includes GGEZ (login auto-launcher) which older lists missed.
pub fn release_steelseries_locks() -> String {
    let images = [
        "SteelSeriesGGEZ",
        "SteelSeriesGG",
        "SteelSeriesGGClient",
        "SteelSeriesEngine",
        "SteelSeriesEngine3",
        "SteelSeriesPrism",
        "GGClient",
        "GGWave",
        "SSOverlay",
    ];
    let mut killed = Vec::new();
    for name in images {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/IM", &format!("{name}.exe"), "/F", "/T"]);
        if crate::win_cmd::hidden(cmd)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            killed.push(name);
        }
    }
    if killed.is_empty() {
        "no SteelSeries GG lock to clear".into()
    } else {
        format!("released SteelSeries lock: {}", killed.join(", "))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_vid_pid() {
        let row = super::parse_hid_path(r"USB\VID_3434&PID_0B60\6&123").unwrap();
        assert_eq!(row.vendor_id, "3434");
        assert_eq!(row.product_id, "0b60");
        assert!(row.name.contains("Keychron"));
        let ss = super::parse_hid_path(r"HID\VID_1038&PID_1A00\1").unwrap();
        assert_eq!(ss.product_id, "1a00");
        assert!(ss.name.contains("Arena"));
    }
}
