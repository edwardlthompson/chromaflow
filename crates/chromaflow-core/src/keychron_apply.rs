//! Keychron Q6 HE VIA paint. Uniform SOLID; per-key Direct in 9-LED batches. No PWM.

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::keychron_preview::{
    drop_session, first_hsv, hsv_close, hue_close, led_count, open_via, with_via_lock, xfer, CMD,
    PACKET, SET_COLOR,
};

const SET_TYPE: u8 = 0x08;
/// Keychron 0xA8/0x08: 0 = static per-key HSV, 1 = breathing (luminance to 0).
const TYPE_SOLID: u8 = 0;
const SOLID_FX: u8 = 1;
const CUSTOM_FX: u8 = 0x17;
const SPEED_STILL: u8 = 0;
const SET_BATCH: u8 = 9;
static DIRECT: AtomicBool = AtomicBool::new(false);
static SOLID: AtomicBool = AtomicBool::new(false);
static LAST_FILL: AtomicU32 = AtomicU32::new(u32::MAX);
const FILL_N: usize = 108;

pub fn rgb_to_hsv(rgb: [u8; 3]) -> [u8; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let delta = max.saturating_sub(min);
    if max == 0 {
        return [0, 0, 0];
    }
    let s = ((u16::from(delta) * 255) / u16::from(max)) as u8;
    if delta == 0 {
        return [0, s, v];
    }
    let d = i16::from(delta);
    let h = if max == r {
        43i16 * (i16::from(g) - i16::from(b)) / d
    } else if max == g {
        85 + 43i16 * (i16::from(b) - i16::from(r)) / d
    } else {
        171 + 43i16 * (i16::from(r) - i16::from(g)) / d
    };
    [h.rem_euclid(256) as u8, s, v]
}

pub fn via_solid_packets(rgb: [u8; 3]) -> Vec<Vec<u8>> {
    let hsv = rgb_to_hsv(rgb);
    vec![
        vec![0x07, 0x03, 0x01, 255],
        vec![0x07, 0x03, 0x02, SOLID_FX],
        vec![0x07, 0x03, 0x03, SPEED_STILL],
        vec![0x07, 0x03, 0x04, hsv[0], hsv[1]],
    ]
}

pub fn via_direct_packets() -> Vec<Vec<u8>> {
    vec![
        vec![0x07, 0x03, 0x01, 255],
        vec![0x07, 0x03, 0x02, CUSTOM_FX],
        vec![0x07, 0x03, 0x03, SPEED_STILL],
        vec![CMD, SET_TYPE, TYPE_SOLID],
    ]
}

pub fn via_effect_get() -> [u8; 3] {
    [0x08, 0x03, 0x02]
}

pub fn set_report(start: u16, colors: &[[u8; 3]]) -> Vec<u8> {
    let take = colors.len().min(usize::from(SET_BATCH));
    let mut data = vec![CMD, SET_COLOR, start as u8, take as u8];
    for rgb in colors.iter().take(take) {
        data.extend_from_slice(&rgb_to_hsv(*rgb));
    }
    data
}

pub fn set_color(rgb: [u8; 3]) -> Result<String, String> {
    drop_session();
    DIRECT.store(false, Ordering::Relaxed);
    SOLID.store(false, Ordering::Relaxed);
    LAST_FILL.store(u32::MAX, Ordering::Relaxed);
    with_via(|file| {
        let want = rgb_to_hsv(rgb);
        for _ in 0..3 {
            enter_solid(file, rgb)?;
            if via_get(file, 2) == Some(SOLID_FX) && via_get(file, 4).is_some_and(|h| hue_close(h, want[0])) {
                SOLID.store(true, Ordering::Relaxed);
                return Ok("set Keychron SOLID via VIA".into());
            }
        }
        enter_solid(file, rgb)?;
        if via_get(file, 2) == Some(SOLID_FX) {
            SOLID.store(true, Ordering::Relaxed);
            return Ok("set Keychron SOLID via VIA".into());
        }
        Err("Keychron SOLID readback failed".into())
    })
}

pub fn poll_effect() -> Option<u8> {
    drop_session();
    with_via(|file| Ok(via_get(file, 2))).ok().flatten()
}

pub fn poll_hue() -> Option<u8> {
    drop_session();
    with_via(|file| Ok(via_get(file, 4))).ok().flatten()
}

pub fn set_hue(h: u8, s: u8) -> Result<String, String> {
    drop_session();
    DIRECT.store(false, Ordering::Relaxed);
    LAST_FILL.store(u32::MAX, Ordering::Relaxed);
    with_via(|file| {
        if !SOLID.load(Ordering::Relaxed) {
            for pkt in via_solid_packets([255, 0, 0]) {
                let _ = via_cmd(file, &pkt);
            }
            SOLID.store(true, Ordering::Relaxed);
        }
        via_cmd(file, &[0x07, 0x03, 0x04, h, s]).ok_or_else(|| "Keychron hue failed".to_string())?;
        Ok("set Keychron SOLID hue".into())
    })
}

pub fn set_led(led: u16, rgb: [u8; 3]) -> Result<String, String> {
    drop_session();
    LAST_FILL.store(u32::MAX, Ordering::Relaxed);
    with_via(|file| {
        DIRECT.store(false, Ordering::Relaxed);
        enter_direct(file)?;
        DIRECT.store(true, Ordering::Relaxed);
        write_a8(file, led, &[rgb])?;
        Ok(format!("set Keychron LED {led} via VIA"))
    })
}

pub fn set_hex_colors(hexes: &[String]) -> Result<String, String> {
    if hexes.is_empty() {
        return Err("color must be RRGGBB".into());
    }
    let mut cols = Vec::new();
    for h in hexes {
        cols.push(crate::lighting_apply::parse_rrggbb(h)?);
    }
    LAST_FILL.store(u32::MAX, Ordering::Relaxed);
    with_via(|file| {
        let n = led_count(file).unwrap_or(108).clamp(1, 255);
        let want = rgb_to_hsv(cols[0]);
        let fill = if cols.len() == 1 {
            vec![cols[0]; usize::from(n)]
        } else {
            cols
        };
        paint_all(file, &fill)?;
        let _ = first_hsv(file).is_some_and(|got| hsv_close(got, want));
        Ok(format!("set Keychron {} LEDs via VIA", fill.len()))
    })
}

pub fn set_fill(rgb: [u8; 3]) -> Result<String, String> {
    let packed = pack_hsv(rgb_to_hsv(rgb));
    if DIRECT.load(Ordering::Relaxed) && LAST_FILL.load(Ordering::Relaxed) == packed {
        return Ok("Keychron Direct fill unchanged".into());
    }
    with_via(|file| {
        paint_all(file, &[rgb; FILL_N])?;
        LAST_FILL.store(packed, Ordering::Relaxed);
        Ok("set Keychron Direct fill".into())
    })
}

fn pack_hsv(hsv: [u8; 3]) -> u32 {
    u32::from_be_bytes([0, hsv[0], hsv[1], hsv[2]])
}

fn paint_all(file: &mut std::fs::File, cols: &[[u8; 3]]) -> Result<(), String> {
    if !DIRECT.load(Ordering::Relaxed) {
        enter_direct(file)?;
        DIRECT.store(true, Ordering::Relaxed);
    }
    let mut start = 0u16;
    while (start as usize) < cols.len() {
        let end = (start as usize + usize::from(SET_BATCH)).min(cols.len());
        write_a8(file, start, &cols[start as usize..end])?;
        start = end as u16;
    }
    Ok(())
}

fn enter_solid(file: &mut std::fs::File, rgb: [u8; 3]) -> Result<(), String> {
    for pkt in via_solid_packets(rgb) {
        via_cmd(file, &pkt).ok_or_else(|| "Keychron SOLID failed".to_string())?;
    }
    if via_get(file, 2) != Some(SOLID_FX) {
        for pkt in via_solid_packets(rgb) {
            via_cmd(file, &pkt).ok_or_else(|| "Keychron SOLID failed".to_string())?;
        }
    }
    let hsv = rgb_to_hsv(rgb);
    via_cmd(file, &[0x07, 0x03, 0x04, hsv[0], hsv[1]]).ok_or_else(|| "Keychron SOLID failed".to_string())?;
    Ok(())
}

fn enter_direct(file: &mut std::fs::File) -> Result<(), String> {
    SOLID.store(false, Ordering::Relaxed);
    for pkt in via_direct_packets() {
        if pkt.first() == Some(&CMD) {
            xfer(file, SET_TYPE, &[TYPE_SOLID]).ok_or_else(|| "Keychron Direct type failed".to_string())?;
        } else {
            via_cmd(file, &pkt).ok_or_else(|| "Keychron custom failed".to_string())?;
        }
    }
    Ok(())
}

fn write_a8(file: &mut std::fs::File, start: u16, colors: &[[u8; 3]]) -> Result<(), String> {
    let payload = set_report(start, colors);
    xfer(file, payload[1], &payload[2..]).ok_or_else(|| "Keychron VIA SET failed".to_string())?;
    let _ = file.flush();
    Ok(())
}

fn via_cmd(file: &mut std::fs::File, data: &[u8]) -> Option<()> {
    let mut buf = [0u8; PACKET];
    buf[1..1 + data.len()].copy_from_slice(data);
    file.write_all(&buf).ok()?;
    let mut resp = [0u8; 32];
    let _ = file.read(&mut resp);
    Some(())
}

fn via_get(file: &mut std::fs::File, id: u8) -> Option<u8> {
    let pkt = via_effect_get();
    let mut buf = [0u8; PACKET];
    buf[1] = pkt[0];
    buf[2] = pkt[1];
    buf[3] = id;
    file.write_all(&buf).ok()?;
    let mut resp = [0u8; 32];
    let n = file.read(&mut resp).ok()?;
    (n > 3).then_some(resp[3])
}

fn with_via<T>(f: impl FnOnce(&mut std::fs::File) -> Result<T, String>) -> Result<T, String> {
    with_via_lock(|| {
        let mut file = open_via().ok_or_else(|| "Keychron VIA hidraw not found".to_string())?;
        f(&mut file)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn set_report_layout_and_hsv() {
        assert_eq!(super::rgb_to_hsv([255, 0, 0]), [0, 255, 255]);
        let p = super::set_report(0, &[[255, 0, 255]]);
        assert_eq!(p[0], 0xa8);
        assert_eq!(p[1], 0x0a);
        assert_eq!(p[2], 0);
        assert_eq!(p[3], 1);
        assert_eq!(&p[4..7], &super::rgb_to_hsv([255, 0, 255]));
        let nine = super::set_report(0, &[[255, 0, 0]; 9]);
        assert_eq!(nine[3], 9);
        assert_eq!(nine.len(), 4 + 9 * 3);
        let d = super::via_direct_packets();
        assert_eq!(d[1], vec![0x07, 0x03, 0x02, 0x17]);
        assert_eq!(d[2], vec![0x07, 0x03, 0x03, 0]);
        assert_eq!(d[3], vec![0xa8, 0x08, 0]);
        let s = super::via_solid_packets([0, 255, 0]);
        assert_eq!(s[1], vec![0x07, 0x03, 0x02, 1]);
        assert_eq!(s[2], vec![0x07, 0x03, 0x03, 0]);
        assert_eq!(s[3][0..3], [0x07, 0x03, 0x04]);
        assert_eq!(super::via_effect_get(), [0x08, 0x03, 0x02]);
        assert!(super::set_hex_colors(&[]).is_err());
        assert_eq!(super::pack_hsv([0, 255, 255]), 0x00ff_ff);
        assert_eq!(super::FILL_N, 108);
    }
}
