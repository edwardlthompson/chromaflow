//! SteelSeries Arena 7 lighting. Vendor HID output report 0x06 only. No PWM.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const VID: &str = "1038";
const PID: &str = "1a00";

pub fn set_color(rgb: [u8; 3]) -> Result<String, String> {
    write_report(&color_report(rgb))
}

pub fn set_zones(hexes: &[String]) -> Result<String, String> {
    if hexes.is_empty() {
        return Err("color must be RRGGBB".into());
    }
    let mut cols = [[0u8; 3]; 4];
    for i in 0..4 {
        let raw = hexes.get(i).unwrap_or(&hexes[0]);
        cols[i] = crate::lighting_apply::parse_rrggbb(raw)?;
    }
    write_report(&zones_report(&cols))?;
    Ok("set Arena 7 4 zones".into())
}

fn write_report(report: &[u8; 64]) -> Result<String, String> {
    let path = vendor_hidraw().ok_or_else(|| "Arena 7 vendor hidraw not found".to_string())?;
    let mut f = OpenOptions::new()
        .write(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(report).map_err(|e| e.to_string())?;
    Ok(format!("set Arena 7 via {}", path.display()))
}

/// 64-byte output report (ID 0x06). Layout from the public Prismatic Arena 7 notes.
pub fn color_report(rgb: [u8; 3]) -> [u8; 64] {
    zones_report(&[rgb, rgb, rgb, rgb])
}

pub fn zones_report(cols: &[[u8; 3]; 4]) -> [u8; 64] {
    let mut r = [0u8; 64];
    r[0] = 0x06;
    r[1] = 0xa1;
    for z in 0..4 {
        let rgb = cols[z];
        let o = 2 + z * 6;
        r[o] = rgb[0];
        r[o + 1] = rgb[1];
        r[o + 2] = rgb[2];
        r[o + 3] = 0x01;
        r[o + 4] = 0x1e;
        r[o + 5] = 10;
    }
    r[26] = 0x0f;
    r
}

fn vendor_hidraw() -> Option<PathBuf> {
    let sys = crate::lighting::hidraw_sys_root();
    let rd = fs::read_dir(&sys).ok()?;
    for ent in rd.flatten() {
        let name = ent.file_name();
        let name = name.to_str()?;
        if !is_arena_vendor(&sys, name) {
            continue;
        }
        return Some(PathBuf::from("/dev").join(name));
    }
    None
}

fn is_arena_vendor(sys: &Path, name: &str) -> bool {
    let text = fs::read_to_string(sys.join(name).join("device/uevent")).unwrap_or_default();
    let hid = text.lines().find_map(|l| l.strip_prefix("HID_ID=")).unwrap_or("");
    let parts: Vec<&str> = hid.split(':').collect();
    if parts.len() < 3 {
        return false;
    }
    let vid = format!("{:0>4}", parts[1].trim().trim_start_matches('0').to_ascii_lowercase());
    let pid = format!("{:0>4}", parts[2].trim().trim_start_matches('0').to_ascii_lowercase());
    if vid != VID || pid != PID {
        return false;
    }
    let desc = fs::read(sys.join(name).join("device/report_descriptor")).unwrap_or_default();
    desc.len() >= 3 && desc[0] == 0x06 && desc[1] == 0xc0 && desc[2] == 0xff
}

#[cfg(test)]
mod tests {
    use super::{color_report, zones_report};

    #[test]
    fn static_report_layout() {
        let r = color_report([0x00, 0xe5, 0xff]);
        assert_eq!(r[0], 0x06);
        assert_eq!(r[1], 0xa1);
        assert_eq!(&r[2..8], &[0x00, 0xe5, 0xff, 0x01, 0x1e, 10]);
        assert_eq!(&r[20..26], &[0x00, 0xe5, 0xff, 0x01, 0x1e, 10]);
        assert_eq!(r[26], 0x0f);
        assert!(r[27..].iter().all(|b| *b == 0));
        let z = zones_report(&[[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]]);
        assert_eq!(&z[2..5], &[255, 0, 0]);
        assert_eq!(&z[8..11], &[0, 255, 0]);
        assert!(super::set_zones(&[]).is_err());
    }
}
