//! Auto-size OpenRGB addressable zones so the engine never asks the user.

use crate::openrgb_proto::{
    close, open_client, read_pkt, request_data, write_pkt, COUNT, RESIZE,
};

/// Default LED count for ARGB headers when OpenRGB cannot detect chain length.
const DEFAULT_ARGB: u32 = 32;
const ZONE_LINEAR: u32 = 1;

fn pick_size(min: u32, max: u32) -> u32 {
    if min <= DEFAULT_ARGB && DEFAULT_ARGB <= max {
        return DEFAULT_ARGB;
    }
    if max >= min && max > 0 {
        return max.min(120).max(min);
    }
    DEFAULT_ARGB
}

/// (zone_index, leds_min, leds_max) for linear zones with leds_count == 0.
pub fn list_resizable_at(data: &[u8], proto: u32) -> Result<Vec<(u32, u32, u32)>, String> {
    let mut i = 0usize;
    for _ in 0..6 {
        skip_str(data, &mut i)?;
    }
    let num_modes = u16_at(data, &mut i)?;
    if num_modes > 256 {
        return Err("too many modes".into());
    }
    let mode_skip = if proto >= 3 { 48 } else { 36 };
    for _ in 0..num_modes {
        skip_str(data, &mut i)?;
        skip(data, &mut i, mode_skip)?;
        let colors = u16_at(data, &mut i)?;
        if colors > 256 {
            return Err("mode colors".into());
        }
        skip(data, &mut i, 4 * usize::from(colors))?;
    }
    let num_zones = u16_at(data, &mut i)?;
    if num_zones > 64 {
        return Err("too many zones".into());
    }
    let mut out = Vec::new();
    for zi in 0..num_zones {
        skip_str(data, &mut i)?;
        let ztype = u32_at(data, &mut i)?;
        let leds_min = u32_at(data, &mut i)?;
        let leds_max = u32_at(data, &mut i)?;
        let leds_count = u32_at(data, &mut i)?;
        let matrix = u16_at(data, &mut i)? as usize;
        if matrix > 1_048_576 {
            return Err("matrix".into());
        }
        skip(data, &mut i, matrix)?;
        if ztype == ZONE_LINEAR && leds_count == 0 && leds_max > 0 {
            out.push((zi as u32, leds_min, leds_max));
        }
    }
    Ok(out)
}

fn resize_zone(stream: &mut std::net::TcpStream, idx: u32, zi: u32, size: u32) -> Result<(), String> {
    let mut body = [0u8; 8];
    body[0..4].copy_from_slice(&(zi as i32).to_le_bytes());
    body[4..8].copy_from_slice(&(size as i32).to_le_bytes());
    write_pkt(stream, idx, RESIZE, &body)
}

/// After SDK is up: resize every zero-sized linear zone across all controllers.
pub fn auto_configure_all() -> Result<String, String> {
    let resized = resize_pass()?;
    // One immediate retry for controllers that need a second RESIZE after the first.
    let again = resize_pass().unwrap_or(0);
    Ok(format!(
        "auto-sized {} addressable zone(s) (default {DEFAULT_ARGB} LEDs)",
        resized + again
    ))
}

fn resize_pass() -> Result<u32, String> {
    let _sdk = crate::openrgb_proto::lock_sdk();
    let (mut stream, proto) = open_client()?;
    write_pkt(&mut stream, 0, COUNT, &[])?;
    let payload = read_pkt(&mut stream)?;
    if payload.len() < 4 {
        close(&mut stream);
        return Err("short count".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count")?) as usize;
    let mut resized = 0u32;
    for idx in 0..n.min(64) {
        let Ok(data) = request_data(&mut stream, idx as u32, proto) else {
            continue;
        };
        let zones = list_resizable_at(&data, proto).unwrap_or_default();
        for (zi, min, max) in zones {
            if resize_zone(&mut stream, idx as u32, zi, pick_size(min, max)).is_ok() {
                resized += 1;
            }
        }
    }
    close(&mut stream);
    Ok(resized)
}

/// How many linear zones still report leds_count == 0 (dialog risk).
pub fn zero_led_linear_zones() -> u32 {
    let _sdk = crate::openrgb_proto::lock_sdk();
    let Ok((mut stream, proto)) = open_client() else {
        return 0;
    };
    if write_pkt(&mut stream, 0, COUNT, &[]).is_err() {
        close(&mut stream);
        return 0;
    }
    let Ok(payload) = read_pkt(&mut stream) else {
        close(&mut stream);
        return 0;
    };
    if payload.len() < 4 {
        close(&mut stream);
        return 0;
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().unwrap_or([0; 4])) as usize;
    let mut left = 0u32;
    for idx in 0..n.min(64) {
        let Ok(data) = request_data(&mut stream, idx as u32, proto) else {
            continue;
        };
        left += list_resizable_at(&data, proto).unwrap_or_default().len() as u32;
    }
    close(&mut stream);
    left
}

fn skip_str(data: &[u8], i: &mut usize) -> Result<(), String> {
    let n = u16_at(data, i)? as usize;
    skip(data, i, n)
}

fn skip(data: &[u8], i: &mut usize, n: usize) -> Result<(), String> {
    let end = i.checked_add(n).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    *i = end;
    Ok(())
}

fn u16_at(data: &[u8], i: &mut usize) -> Result<u16, String> {
    let end = i.checked_add(2).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    let v = u16::from_le_bytes(data[*i..end].try_into().map_err(|_| "u16")?);
    *i = end;
    Ok(v)
}

fn u32_at(data: &[u8], i: &mut usize) -> Result<u32, String> {
    let end = i.checked_add(4).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    let v = u32::from_le_bytes(data[*i..end].try_into().map_err(|_| "u32")?);
    *i = end;
    Ok(v)
}

#[cfg(test)]
mod tests {
    #[test]
    fn pick_prefers_default_inside_range() {
        assert_eq!(super::pick_size(0, 120), 32);
        assert_eq!(super::pick_size(1, 20), 20);
        assert_eq!(super::pick_size(40, 80), 80);
    }
}
