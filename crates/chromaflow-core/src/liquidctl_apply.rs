//! Fusion color via fusion-hid.py; liquidctl analog is optional. Never vendors rgb_fusion2.py.

use std::process::Command;

const ALL: [&str; 8] = [
    "led1", "led2", "led3", "led4", "led5", "led6", "led7", "led8",
];
const BOARD: [&str; 3] = ["led1", "led3", "led4"];
const AIO: [&str; 5] = ["led2", "led5", "led6", "led7", "led8"];

pub fn channels(device: &str) -> &'static [&'static str] {
    let n = device.to_ascii_lowercase();
    if n.contains("motherboard") {
        &BOARD
    } else if n.contains("aio") {
        &AIO
    } else {
        &ALL
    }
}

pub fn hid_kind(device: &str) -> &'static str {
    let n = device.to_ascii_lowercase();
    if n.contains("motherboard") {
        "soft"
    } else if n.contains("aio") {
        "digital"
    } else {
        "uniform"
    }
}

pub fn style(mode: &str) -> &'static str {
    let m = mode.to_ascii_lowercase();
    if m.contains("breath") || m.contains("pulse") {
        "pulse"
    } else if m.contains("flash") || m.contains("strobe") {
        "flash"
    } else {
        "fixed"
    }
}

pub fn set_fusion_sync(hex: &str) -> Result<String, String> {
    set_device("CPU AIO", hex, "fixed")
}

pub fn set_fusion(hex: &str) -> Result<String, String> {
    set_device("CPU AIO", hex, "fixed")
}

pub fn set_device(device: &str, hex: &str, mode: &str) -> Result<String, String> {
    let st = style(mode);
    if st == "fixed" {
        if let Ok(msg) = hid_set(hid_kind(device), hex) {
            let _ = set_on(channels(device), st, hex);
            return Ok(msg);
        }
    }
    set_on(channels(device), st, hex)
}

pub fn set_led(device: &str, led: u16, hex: &str) -> Result<String, String> {
    let chans = channels(device);
    let Some(ch) = chans.get(usize::from(led)).copied() else {
        return Err("Fusion LED".into());
    };
    set_on(&[ch], "fixed", hex)
}

fn hid_set(kind: &str, hex: &str) -> Result<String, String> {
    crate::fusion_hid::set(kind, hex)
}

fn set_on(chans: &[&str], style: &str, hex: &str) -> Result<String, String> {
    valid_hex(hex)?;
    let bin = bin();
    let mut n = 0u32;
    for ch in chans {
        if run(&bin, &["-m", "Fusion", "set", ch, "color", style, hex]).is_ok() {
            n += 1;
        }
    }
    if n == 0 {
        return Err("liquidctl Fusion failed".into());
    }
    Ok(format!("set Fusion {style} on {n} channels"))
}

fn valid_hex(hex: &str) -> Result<(), String> {
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("color must be RRGGBB".into());
    }
    Ok(())
}

fn bin() -> String {
    std::env::var("CHROMAFLOW_LIQUIDCTL").unwrap_or_else(|_| "liquidctl".into())
}

fn run(bin: &str, args: &[&str]) -> Result<(), String> {
    let mut cmd = vec!["3", bin];
    cmd.extend_from_slice(args);
    let out = Command::new("timeout")
        .args(&cmd)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{channels, hid_kind, set_fusion, style};

    #[test]
    fn rejects_empty_hex() {
        assert!(super::set_fusion_sync("").is_err());
        assert!(set_fusion("").is_err());
        assert!(set_fusion("00").is_err());
        assert_eq!(
            channels("CPU AIO"),
            &["led2", "led5", "led6", "led7", "led8"]
        );
        assert_eq!(channels("Motherboard Fusion"), &["led1", "led3", "led4"]);
        assert_eq!(hid_kind("Motherboard Fusion"), "soft");
        assert_eq!(hid_kind("CPU AIO"), "digital");
        assert_eq!(hid_kind("sync"), "uniform");
        assert_eq!(style("Cycle All"), "fixed");
        assert_eq!(style("Breathing"), "pulse");
        assert_eq!(style("sync"), "fixed");
    }
}
