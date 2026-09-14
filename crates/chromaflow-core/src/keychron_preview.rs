//! Poll Keychron Q6 HE per-key HSV via VIA raw HID. No PWM. No OpenRGB C++.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread;

const VID: &str = "3434";
pub(crate) const CMD: u8 = 0xa8;
pub(crate) const GET_COUNT: u8 = 0x05;
pub(crate) const GET_COLOR: u8 = 0x09;
pub(crate) const SET_COLOR: u8 = 0x0a;
pub(crate) const PACKET: usize = 33;
pub(crate) const BATCH: u8 = 9;
static VIA: Mutex<()> = Mutex::new(());

pub fn with_via_lock<T>(f: impl FnOnce() -> T) -> T {
    let _g = VIA.lock().unwrap_or_else(|p| p.into_inner());
    f()
}

enum Msg {
    Kick,
}

static LAST: Mutex<Vec<String>> = Mutex::new(Vec::new());
static TX: Mutex<Option<Sender<Msg>>> = Mutex::new(None);

pub fn kick() {
    ensure();
    if let Some(tx) = TX.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
        let _ = tx.send(Msg::Kick);
    }
}

pub fn latest() -> Vec<String> {
    kick();
    LAST.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

pub fn drop_session() {
    *TX.lock().unwrap_or_else(|p| p.into_inner()) = None;
}

fn ensure() {
    let mut slot = TX.lock().unwrap_or_else(|p| p.into_inner());
    if slot.is_some() {
        return;
    }
    let (tx, rx) = mpsc::channel();
    *slot = Some(tx);
    let _ = thread::Builder::new()
        .name("keychron-leds".into())
        .spawn(move || {
            let mut file = match open_via() {
                Some(f) => f,
                None => return,
            };
            while rx.recv().is_ok() {
                while rx.try_recv().is_ok() {}
                let cols = with_via_lock(|| read_all(&mut file));
                if let Some(cols) = cols {
                    *LAST.lock().unwrap_or_else(|p| p.into_inner()) = cols;
                }
            }
        });
}

pub(crate) fn open_via() -> Option<File> {
    let sys = crate::lighting::hidraw_sys_root();
    let rd = fs::read_dir(&sys).ok()?;
    for ent in rd.flatten() {
        let name = ent.file_name();
        let name = name.to_str()?;
        if !is_via(&sys, name) {
            continue;
        }
        return OpenOptions::new()
            .read(true)
            .write(true)
            .open(PathBuf::from("/dev").join(name))
            .ok();
    }
    None
}

fn is_via(sys: &Path, name: &str) -> bool {
    let text = fs::read_to_string(sys.join(name).join("device/uevent")).unwrap_or_default();
    let hid = text
        .lines()
        .find_map(|l| l.strip_prefix("HID_ID="))
        .unwrap_or("");
    let parts: Vec<&str> = hid.split(':').collect();
    if parts.len() < 3 {
        return false;
    }
    let vid = format!(
        "{:0>4}",
        parts[1].trim().trim_start_matches('0').to_ascii_lowercase()
    );
    if vid != VID {
        return false;
    }
    let desc = fs::read(sys.join(name).join("device/report_descriptor")).unwrap_or_default();
    desc.len() >= 3 && desc[0] == 0x06 && desc[1] == 0x60 && desc[2] == 0xff
}

pub fn poll_colors() -> Option<Vec<String>> {
    with_via_lock(|| {
        let mut file = open_via()?;
        read_all(&mut file)
    })
}

pub fn hue_close(a: u8, b: u8) -> bool {
    let d = u16::from(a.abs_diff(b));
    d.min(256 - d) <= 16
}

pub fn hsv_close(got: [u8; 3], want: [u8; 3]) -> bool {
    hue_close(got[0], want[0]) && got[1].abs_diff(want[1]) <= 48 && got[2].abs_diff(want[2]) <= 48
}

pub(crate) fn first_hsv(file: &mut File) -> Option<[u8; 3]> {
    let resp = xfer(file, GET_COLOR, &[0, 3])?;
    if resp.len() < 6 || resp[0] != CMD || resp[1] != GET_COLOR {
        return None;
    }
    Some([resp[3], resp[4], resp[5]])
}

fn read_all(file: &mut File) -> Option<Vec<String>> {
    let n = led_count(file)?;
    let mut out = Vec::with_capacity(usize::from(n));
    let mut start = 0u16;
    while start < n {
        let take = BATCH.min((n - start) as u8);
        let resp = xfer(file, GET_COLOR, &[start as u8, take])?;
        if resp.len() < 6 || resp[0] != CMD || resp[1] != GET_COLOR {
            return None;
        }
        let mut i = 3usize;
        for _ in 0..take {
            if i + 2 >= resp.len() {
                break;
            }
            out.push(hsv255(resp[i], resp[i + 1], resp[i + 2]));
            i += 3;
        }
        start = start.saturating_add(u16::from(take));
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

pub(crate) fn led_count(file: &mut File) -> Option<u16> {
    let resp = xfer(file, GET_COUNT, &[])?;
    if resp.len() < 4 || resp[0] != CMD {
        return None;
    }
    let n = u16::from(resp[3]);
    (n > 0 && n <= 255).then_some(n)
}

pub(crate) fn xfer(file: &mut File, sub: u8, data: &[u8]) -> Option<Vec<u8>> {
    let mut buf = [0u8; PACKET];
    buf[1] = CMD;
    buf[2] = sub;
    buf[3..3 + data.len()].copy_from_slice(data);
    file.write_all(&buf).ok()?;
    let mut resp = [0u8; 32];
    let n = file.read(&mut resp).ok()?;
    Some(resp[..n].to_vec())
}

pub fn hsv255(h: u8, s: u8, v: u8) -> String {
    let rgb = hsv_to_rgb(h, s, v);
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

fn hsv_to_rgb(h: u8, s: u8, v: u8) -> [u8; 3] {
    if s == 0 {
        return [v, v, v];
    }
    let region = u16::from(h) / 43;
    let rem = (u16::from(h) - region * 43) * 6;
    let p = u8::try_from(u16::from(v) * (255 - u16::from(s)) / 255).unwrap_or(0);
    let q = u8::try_from(u16::from(v) * (255 - u16::from(s) * rem / 255) / 255).unwrap_or(0);
    let t =
        u8::try_from(u16::from(v) * (255 - u16::from(s) * (255 - rem) / 255) / 255).unwrap_or(0);
    match region {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn red_hsv_is_ff0000() {
        assert_eq!(super::hsv255(0, 255, 255), "#ff0000");
        assert!(super::hue_close(0, 8));
        assert!(super::hue_close(0, 250));
        assert!(!super::hue_close(0, 40));
        assert!(super::hsv_close([0, 255, 255], [8, 240, 250]));
        assert!(!super::hsv_close([0, 255, 255], [0, 0, 255]));
    }
}
