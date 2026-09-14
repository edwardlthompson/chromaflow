//! HID/USB RGB candidates from sysfs. Never opens hidraw or writes LEDs.

use crate::types::{DevNode, HidRgb};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const KNOWN: &[(&str, &str, &str, &str)] = &[
    (
        "048d",
        "5702",
        "motherboard",
        "Gigabyte RGB Fusion 2.0 (ARGB headers)",
    ),
    ("1038", "1856", "mouse", "SteelSeries Prime Neo Noir"),
    ("3434", "0b60", "keyboard", "Keychron Q6 HE"),
    ("1038", "1a00", "speakers", "SteelSeries Arena 7"),
];

pub fn hidraw_sys_root() -> PathBuf {
    std::env::var("CHROMAFLOW_HIDRAW_SYS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/class/hidraw"))
}

fn sysfs_text(path: &Path) -> String {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn usb_field(sys_root: &Path, base: &Path, name: &str) -> String {
    sysfs_text(&sys_root.join(base).join("device/../../").join(name))
}

pub fn hid_serial(sys_root: &Path, hidraw: &str) -> String {
    usb_field(sys_root, Path::new(hidraw), "serial")
}

pub fn scan(sys_root: &Path, nodes: &[DevNode]) -> Vec<HidRgb> {
    let mut best: BTreeMap<(String, String), HidRgb> = BTreeMap::new();
    for node in nodes {
        let Some(base) = Path::new(&node.path).file_name() else {
            continue;
        };
        let Ok(text) = fs::read_to_string(sys_root.join(base).join("device/uevent")) else {
            continue;
        };
        let (vid, pid, hid_name) = parse_uevent(&text);
        let Some((kind, name)) = classify(&vid, &pid, &hid_name) else {
            continue;
        };
        best.entry((vid.clone(), pid.clone())).or_insert(HidRgb {
            path: node.path.clone(),
            name,
            kind,
            vendor_id: vid,
            product_id: pid,
            readable: node.readable,
            hid_name: hid_name.clone(),
            manufacturer: usb_field(sys_root, Path::new(base), "manufacturer"),
            product: usb_field(sys_root, Path::new(base), "product"),
        });
    }
    scan_usb_fusion(&mut best);
    best.into_values().take(32).collect()
}

fn usb_bus_root() -> PathBuf {
    std::env::var("CHROMAFLOW_USB_SYS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/bus/usb/devices"))
}

fn scan_usb_fusion(best: &mut BTreeMap<(String, String), HidRgb>) {
    let usb = usb_bus_root();
    let Ok(entries) = fs::read_dir(&usb) else {
        return;
    };
    for ent in entries.flatten() {
        let dir = ent.path();
        let vid = pad4(&sysfs_text(&dir.join("idVendor")));
        let pid = pad4(&sysfs_text(&dir.join("idProduct")));
        if vid != "048d" || pid != "5702" || best.contains_key(&(vid.clone(), pid.clone())) {
            continue;
        }
        let Some((kind, name)) = classify(&vid, &pid, "") else {
            continue;
        };
        best.insert(
            (vid.clone(), pid.clone()),
            HidRgb {
                path: dir.display().to_string(),
                name,
                kind,
                vendor_id: vid,
                product_id: pid,
                readable: false,
                hid_name: String::new(),
                manufacturer: sysfs_text(&dir.join("manufacturer")),
                product: sysfs_text(&dir.join("product")),
            },
        );
    }
}

const LIQUID_TTL: Duration = Duration::from_secs(15);
const LIQUID_NEG: Duration = Duration::from_secs(2);

struct LiquidCache {
    ok: Option<(Instant, Vec<String>)>,
    fail_at: Option<Instant>,
}

static LIQUID: Mutex<LiquidCache> = Mutex::new(LiquidCache {
    ok: None,
    fail_at: None,
});

pub fn liquidctl_devices() -> Vec<String> {
    let now = Instant::now();
    {
        let g = LIQUID.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((at, rows)) = &g.ok {
            if now.saturating_duration_since(*at) < LIQUID_TTL {
                return rows.clone();
            }
        }
        if let Some(at) = g.fail_at {
            if now.saturating_duration_since(at) < LIQUID_NEG {
                return g.ok.as_ref().map(|(_, r)| r.clone()).unwrap_or_default();
            }
        }
    }
    let got = match Command::new("liquidctl").arg("list").output() {
        Ok(out) if out.status.success() => {
            Some(parse_liquidctl(&String::from_utf8_lossy(&out.stdout)))
        }
        _ => None,
    };
    let mut g = LIQUID.lock().unwrap_or_else(|p| p.into_inner());
    match got {
        Some(rows) => {
            g.fail_at = None;
            g.ok = Some((Instant::now(), rows.clone()));
            rows
        }
        None => {
            g.fail_at = Some(Instant::now());
            g.ok.as_ref().map(|(_, r)| r.clone()).unwrap_or_default()
        }
    }
}

fn parse_uevent(text: &str) -> (String, String, String) {
    let mut vid = String::new();
    let mut pid = String::new();
    let mut name = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("HID_NAME=") {
            name = rest.trim().to_string();
        }
        if let Some(rest) = line.strip_prefix("HID_ID=") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 3 {
                vid = pad4(parts[1]);
                pid = pad4(parts[2]);
            }
        }
    }
    (vid, pid, name)
}

fn pad4(raw: &str) -> String {
    let hex = raw.trim().trim_start_matches('0').to_ascii_lowercase();
    format!("{:0>4}", if hex.is_empty() { "0" } else { &hex })
}

fn noise(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [
        "ups",
        "microphone",
        "fingerprint",
        "camera",
        "touchpad",
        "wacom",
        "cs201",
        "consumer control",
    ]
    .iter()
    .any(|k| n.contains(k))
}

fn classify(vid: &str, pid: &str, name: &str) -> Option<(String, String)> {
    for (v, p, kind, label) in KNOWN {
        if vid.eq_ignore_ascii_case(v) && pid.eq_ignore_ascii_case(p) {
            return Some(((*kind).into(), (*label).into()));
        }
    }
    if noise(name) {
        return None;
    }
    let n = name.to_ascii_lowercase();
    if vid == "048d" || n.contains("fusion") || n.contains("argb") || n.contains("rgb led") {
        return Some(("motherboard".into(), label(name, "Motherboard ARGB")));
    }
    if vid == "3434" || n.contains("keychron") || (n.contains("keyboard") && n.contains("rgb")) {
        return Some(("keyboard".into(), label(name, "RGB keyboard")));
    }
    if n.contains("arena") || n.contains("speaker") {
        return Some(("speakers".into(), label(name, "RGB speakers")));
    }
    if n.contains("mouse") || n.contains("prime") || n.contains("rival") {
        return Some(("mouse".into(), label(name, "RGB mouse")));
    }
    if n.contains("liquid") || n.contains("radiator") || n.contains(" aio") {
        return Some(("radiator".into(), label(name, "Radiator RGB")));
    }
    Some(("other".into(), label(name, "Other USB HID")))
}

fn label(name: &str, fallback: &str) -> String {
    let t = name.trim();
    if t.is_empty() {
        fallback.into()
    } else {
        t.into()
    }
}

pub fn hid_has_linux_backend(vid: &str, pid: &str, name: &str, sdk: &str, liquid: &str) -> bool {
    let vid = vid.to_ascii_lowercase();
    let pid = pid.to_ascii_lowercase();
    let sdk = sdk.to_ascii_lowercase();
    let liquid = liquid.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    if vid == "1038" && (pid == "1a00" || pid == "1856") {
        return true;
    }
    if vid == "048d" && pid == "5702" {
        return sdk.contains("aorus")
            || sdk.contains("gigabyte")
            || sdk.contains("fusion")
            || liquid.contains("fusion");
    }
    if vid == "3434" && sdk.contains("keychron") {
        return true;
    }
    !name.is_empty() && sdk.contains(&name)
}

fn parse_liquidctl(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("Device #")
                .and_then(|rest| rest.split_once(':'))
                .map(|(_, n)| n.trim().to_string())
                .filter(|n| !n.is_empty())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_uevent_and_noise() {
        assert_eq!(classify("048d", "5702", "ITE").unwrap().0, "motherboard");
        assert_eq!(classify("3434", "0b60", "Q6").unwrap().0, "keyboard");
        assert_eq!(classify("1038", "1856", "Prime").unwrap().0, "mouse");
        assert_eq!(classify("1038", "1a00", "Arena").unwrap().0, "speakers");
        assert!(classify("051d", "0002", "Back-UPS NS").is_none());
        assert!(classify("046d", "0a44", "USB Camera").is_none());
        assert!(classify("0020", "0b21", "Generic CS201").is_none());
        assert!(hid_has_linux_backend(
            "048d",
            "5702",
            "Gigabyte RGB Fusion 2.0",
            "X570S AORUS MASTER",
            ""
        ));
        assert!(hid_has_linux_backend(
            "048d",
            "5702",
            "Fusion",
            "",
            "Gigabyte RGB Fusion 2.0 5702 Controller"
        ));
        assert!(!hid_has_linux_backend("048d", "5702", "Fusion", "", ""));
        assert!(hid_has_linux_backend("1038", "1a00", "Arena 7", "", ""));
        let (v, p, n) = parse_uevent("HID_ID=0003:0000048D:00005702\nHID_NAME=ITE Device\n");
        assert_eq!(
            (v.as_str(), p.as_str(), n.as_str()),
            ("048d", "5702", "ITE Device")
        );
        assert_eq!(
            parse_liquidctl("Device #0: Gigabyte RGB Fusion 2.0 5702 Controller\n"),
            ["Gigabyte RGB Fusion 2.0 5702 Controller"]
        );
        let usb = std::path::PathBuf::from("target/usb-fusion-test");
        let node = usb.join("3-1");
        let _ = std::fs::create_dir_all(&node);
        std::fs::write(node.join("idVendor"), "048d\n").unwrap();
        std::fs::write(node.join("idProduct"), "5702\n").unwrap();
        std::env::set_var("CHROMAFLOW_USB_SYS", &usb);
        let rows = scan(Path::new("/no-hidraw"), &[]);
        assert!(rows.iter().any(|r| r.vendor_id == "048d" && r.product_id == "5702"));
    }
}
