//! SteelSeries Prime Neo wheel LED. HID output 0x62 + save 0x59. No PWM.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const VID: &str = "1038";
const PID: &str = "1856";

pub fn set_color(rgb: [u8; 3]) -> Result<String, String> {
    let path = vendor_hidraw().ok_or_else(|| "Prime Neo vendor hidraw not found".to_string())?;
    let mut f = OpenOptions::new()
        .write(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(&color_report(rgb)).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(50));
    f.write_all(&save_report()).map_err(|e| e.to_string())?;
    Ok(format!("set Prime Neo wheel via {}", path.display()))
}

/// Report ID 0 + command 0x62 0x01 + RGB + documented suffix ending 0xFF.
pub fn color_report(rgb: [u8; 3]) -> [u8; 22] {
    let mut r = [0u8; 22];
    r[1] = 0x62;
    r[2] = 0x01;
    r[3] = rgb[0];
    r[4] = rgb[1];
    r[5] = rgb[2];
    r[21] = 0xff;
    r
}

pub fn save_report() -> [u8; 2] {
    [0x00, 0x59]
}

fn vendor_hidraw() -> Option<PathBuf> {
    let sys = crate::lighting::hidraw_sys_root();
    let rd = fs::read_dir(&sys).ok()?;
    for ent in rd.flatten() {
        let name = ent.file_name();
        let name = name.to_str()?;
        if is_prime_vendor(&sys, name) {
            return Some(PathBuf::from("/dev").join(name));
        }
    }
    None
}

fn is_prime_vendor(sys: &Path, name: &str) -> bool {
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
    #[test]
    fn static_report_layout() {
        let r = super::color_report([0x00, 0x52, 0xff]);
        assert_eq!(r[0], 0x00);
        assert_eq!(&r[1..6], &[0x62, 0x01, 0x00, 0x52, 0xff]);
        assert_eq!(r[21], 0xff);
        assert!(r[6..21].iter().all(|b| *b == 0));
        assert_eq!(super::save_report(), [0x00, 0x59]);
    }
}
