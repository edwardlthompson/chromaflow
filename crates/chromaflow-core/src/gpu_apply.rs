//! MSI RTX 4090 Suprim Liquid X RGB via NVIDIA I2C. No OpenRGB C++. No PWM.
//! Never dump this bus (full SMBus read wedges it EIO). ITE colors do not read back.

use crate::types::RgbDevice;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const VEN: &str = "10de";
const DEV: &str = "2684";
const SVEN: &str = "1462";
const SDEV: &str = "5104";
const ADDR: u8 = 0x68;
const NAME: &str = "MSI GeForce RTX 4090 Suprim Liquid X";
static LAST_BUS: Mutex<Option<u8>> = Mutex::new(None);
static LAST_RGB: Mutex<Option<[u8; 3]>> = Mutex::new(None);

pub fn pci_root() -> PathBuf {
    std::env::var("CHROMAFLOW_PCI_SYS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/bus/pci/devices"))
}

fn hex4(raw: &str) -> String {
    let t = raw.trim().trim_start_matches("0x").to_ascii_lowercase();
    format!("{t:0>4}")
}

pub fn read_ids(dir: &Path) -> Option<[String; 4]> {
    let one = |n: &str| fs::read_to_string(dir.join(n)).ok().map(|s| hex4(&s));
    Some([
        one("vendor")?,
        one("device")?,
        one("subsystem_vendor")?,
        one("subsystem_device")?,
    ])
}

pub fn is_suprim_liquid(ids: &[String; 4]) -> bool {
    ids[0] == VEN && ids[1] == DEV && ids[2] == SVEN && ids[3] == SDEV
}

pub fn present() -> bool {
    let Ok(entries) = fs::read_dir(pci_root()) else {
        return false;
    };
    entries
        .flatten()
        .any(|e| read_ids(&e.path()).is_some_and(|ids| is_suprim_liquid(&ids)))
}

pub fn devices() -> Vec<RgbDevice> {
    if !present() {
        return Vec::new();
    }
    let mut d = RgbDevice::sdk(NAME);
    d.protocol = "NVIDIA I2C 0x68".to_string();
    d.leds = 1;
    vec![d]
}

/// Linux NVIDIA adapter 1 (`i2c-0` here) is the ITE, not a dummy DDC skip.
pub fn prefer_first(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("adapter 1 at") || n.trim().ends_with("adapter 1")
}

fn adapter_name(bus: u8) -> String {
    for p in [
        format!("/sys/bus/i2c/devices/i2c-{bus}/name"),
        format!("/sys/class/i2c-adapter/i2c-{bus}/name"),
    ] {
        if let Ok(s) = fs::read_to_string(p) {
            return s;
        }
    }
    String::new()
}

fn buses() -> Vec<u8> {
    let mut first = Vec::new();
    let mut rest = Vec::new();
    for bus in 0u8..32 {
        let n = adapter_name(bus);
        if !n.to_ascii_lowercase().contains("nvidia") {
            continue;
        }
        if prefer_first(&n) {
            first.push(bus);
        } else {
            rest.push(bus);
        }
    }
    first.extend(rest);
    first
}

/// Firmware rainbow on the ITE. One I2C burst; do not poll this bus from Cycle All.
pub fn set_rainbow() -> Result<String, String> {
    if let Ok(mut g) = LAST_RGB.lock() {
        *g = None;
    }
    write_fx(0x08)
}

pub fn set_color(rgb: [u8; 3]) -> Result<String, String> {
    let mut last = LAST_RGB.lock().unwrap_or_else(|p| p.into_inner());
    let note = write_rgb(rgb, true)?;
    *last = Some(rgb);
    Ok(note)
}

/// Cycle All: RGB1 is the back buffer (not shown). Flip is static `0x13` + save.
/// The first snap still uses the full idle Apply sequence to arm a front buffer.
pub fn set_snap(rgb: [u8; 3]) -> Result<String, String> {
    let Ok(mut last) = LAST_RGB.try_lock() else {
        return Ok("GPU live busy".into());
    };
    if *last == Some(rgb) {
        return Ok("GPU live unchanged".into());
    }
    let note = if last.is_some() {
        match write_next(rgb) {
            Ok(n) => n,
            Err(_) => write_rgb(rgb, true)?,
        }
    } else {
        write_rgb(rgb, true)?
    };
    *last = Some(rgb);
    Ok(note)
}

fn write_next(rgb: [u8; 3]) -> Result<String, String> {
    on_bus(|bus| {
        i2c_sets(bus, &snap_pairs(rgb))?;
        Ok(format!(
            "flip GPU RGB {:02X}{:02X}{:02X} on i2c-{bus}",
            rgb[0], rgb[1], rgb[2]
        ))
    })
}

/// Back buffer RGB1, then re-enter static and save. No idle (that drops the front color).
fn snap_pairs(rgb: [u8; 3]) -> [(u8, u8); 5] {
    [
        (0x30, rgb[0]),
        (0x31, rgb[1]),
        (0x32, rgb[2]),
        (0x22, 0x13),
        (0x3f, 0x00),
    ]
}

fn i2c_sets(bus: u8, pairs: &[(u8, u8)]) -> Result<(), String> {
    let script = pairs
        .iter()
        .map(|(reg, val)| format!("i2cset -y -f {bus} {ADDR:#04x} {reg:#04x} {val:#04x}"))
        .collect::<Vec<_>>()
        .join(" && ");
    let out = Command::new("timeout")
        .args(["0.4", "sh", "-c", &script])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn write_rgb(rgb: [u8; 3], save: bool) -> Result<String, String> {
    on_bus(|bus| {
        paint(bus, rgb, save)?;
        Ok(format!(
            "set GPU RGB {:02X}{:02X}{:02X} on i2c-{bus}",
            rgb[0], rgb[1], rgb[2]
        ))
    })
}

fn write_fx(mode: u8) -> Result<String, String> {
    on_bus(|bus| {
        fx(bus, mode)?;
        Ok(format!("set GPU FX {mode:#04x} on i2c-{bus}"))
    })
}

fn on_bus(f: impl Fn(u8) -> Result<String, String>) -> Result<String, String> {
    if !present() {
        return Err("MSI GPU RGB: PCI 10de:2684/1462:5104 not found".into());
    }
    let mut order = Vec::new();
    if let Ok(g) = LAST_BUS.lock() {
        if let Some(b) = *g {
            order.push(b);
        }
    }
    for b in buses() {
        if !order.contains(&b) {
            order.push(b);
        }
    }
    let mut last_err = "MSI GPU RGB: no NVIDIA I2C bus".to_string();
    for bus in order {
        match f(bus) {
            Ok(note) => {
                if let Ok(mut g) = LAST_BUS.lock() {
                    *g = Some(bus);
                }
                return Ok(note);
            }
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

fn i2c(args: &[&str]) -> Result<(), String> {
    let out = Command::new("timeout")
        .args(["0.4"])
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        thread::sleep(Duration::from_millis(20));
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn paint(bus: u8, rgb: [u8; 3], save: bool) -> Result<(), String> {
    let bus_s = bus.to_string();
    let addr = format!("{ADDR:#04x}");
    let put = |reg: u8, val: u8| {
        i2c(&[
            "i2cset",
            "-y",
            "-f",
            &bus_s,
            &addr,
            &format!("{reg:#04x}"),
            &format!("{val:#04x}"),
        ])
    };
    let block = |reg: u8| {
        i2c(&[
            "i2cset",
            "-y",
            "-f",
            &bus_s,
            &addr,
            &format!("{reg:#04x}"),
            &format!("{:#04x}", rgb[2]),
            &format!("{:#04x}", rgb[1]),
            &format!("{:#04x}", rgb[0]),
            "i",
        ])
    };
    put(0x2e, 0x00)?;
    put(0x22, 0x1c)?;
    put(0x30, rgb[0])?;
    put(0x31, rgb[1])?;
    put(0x32, rgb[2])?;
    put(0x36, 0x64)?;
    let _ = block(0x27);
    let _ = block(0x28);
    let _ = block(0x29);
    put(0x22, 0x13)?;
    if save {
        put(0x3f, 0x00)?;
    }
    Ok(())
}

fn fx(bus: u8, mode: u8) -> Result<(), String> {
    let bus_s = bus.to_string();
    let addr = format!("{ADDR:#04x}");
    let put = |reg: u8, val: u8| {
        i2c(&[
            "i2cset",
            "-y",
            "-f",
            &bus_s,
            &addr,
            &format!("{reg:#04x}"),
            &format!("{val:#04x}"),
        ])
    };
    put(0x2e, 0x00)?;
    put(0x22, 0x1c)?;
    put(0x46, 0x00)?;
    put(0x36, 0x64)?;
    put(0x38, 0x02)?;
    put(0x22, mode)?;
    put(0x3f, 0x00)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn pci_match_prefer_adapter_one() {
        let dir = std::env::temp_dir().join("cf-gpu-pci");
        let _ = fs::create_dir_all(&dir);
        let slot = dir.join("0000:0b:00.0");
        let _ = fs::create_dir_all(&slot);
        fs::write(slot.join("vendor"), "0x10de\n").unwrap();
        fs::write(slot.join("device"), "0x2684\n").unwrap();
        fs::write(slot.join("subsystem_vendor"), "0x1462\n").unwrap();
        fs::write(slot.join("subsystem_device"), "0x5104\n").unwrap();
        std::env::set_var("CHROMAFLOW_PCI_SYS", &dir);
        assert!(present());
        assert_eq!(devices()[0].name, NAME);
        assert_eq!(ADDR, 0x68);
        std::env::remove_var("CHROMAFLOW_PCI_SYS");
        assert!(prefer_first("NVIDIA i2c adapter 1 at b:00.0"));
        assert!(!prefer_first("NVIDIA i2c adapter 11 at b:00.0"));
        assert!(!prefer_first("NVIDIA i2c adapter 6 at b:00.0"));
        assert!(!is_suprim_liquid(&[
            "10de".into(),
            "2684".into(),
            "1462".into(),
            "0000".into()
        ]));
        let p = snap_pairs([0x11, 0x22, 0x33]);
        assert_eq!(p[0], (0x30, 0x11));
        assert_eq!(&p[3..], &[(0x22, 0x13), (0x3f, 0x00)]);
        assert!(!p.iter().any(|&(r, v)| r == 0x22 && v == 0x1c));
    }
}
