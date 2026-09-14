//! UPDATE_MODE and UPDATE_SINGLE_LED on localhost. No PWM.

use std::net::TcpStream;
use std::time::Duration;

use crate::openrgb_parse::{fill_motion, find_mode_at, parse_led_count_at, tune_mode};
use crate::openrgb_proto::{
    close, open_client, parse_name, request_data, write_pkt, COUNT, SET_CUSTOM, UPDATE_LED,
    UPDATE_MODE,
};

pub fn set_mode(name: &str, mode: &str, rgb: [u8; 3]) -> Result<String, String> {
    let label = mode.trim();
    if label.eq_ignore_ascii_case("direct") || label.eq_ignore_ascii_case("custom") {
        return crate::openrgb_apply::set_color(name, rgb);
    }
    crate::openrgb_preview::drop_session();
    let _sdk = crate::openrgb_proto::lock_sdk();
    let (mut stream, proto) = open_client()?;
    let (idx, data) = find_data(&mut stream, name, proto)?;
    write_mode(&mut stream, idx, proto, &data, label, rgb)?;
    std::thread::sleep(Duration::from_millis(80));
    if let Ok(fresh) = request_data(&mut stream, idx, proto) {
        let _ = write_mode(&mut stream, idx, proto, &fresh, label, rgb);
    }
    std::thread::sleep(Duration::from_millis(40));
    close(&mut stream);
    Ok(format!("set {name} mode {label} via OpenRGB SDK"))
}

pub fn set_led(name: &str, led: u16, rgb: [u8; 3]) -> Result<String, String> {
    crate::openrgb_preview::drop_session();
    let _sdk = crate::openrgb_proto::lock_sdk();
    let (mut stream, proto) = open_client()?;
    let (idx, data) = find_data(&mut stream, name, proto)?;
    let leds = parse_led_count_at(&data, proto)?;
    if u32::from(led) >= u32::from(leds) {
        close(&mut stream);
        return Err("LED index out of range".into());
    }
    write_pkt(&mut stream, idx, SET_CUSTOM, &[])?;
    write_pkt(&mut stream, idx, UPDATE_LED, &update_led_body(led, rgb))?;
    std::thread::sleep(Duration::from_millis(40));
    close(&mut stream);
    Ok(format!("set {name} LED {led} via OpenRGB SDK"))
}

pub fn update_mode_body(mode_idx: i32, blob: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(8 + blob.len());
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&mode_idx.to_le_bytes());
    body.extend_from_slice(blob);
    let data_size = u32::try_from(body.len()).unwrap_or(0);
    body[0..4].copy_from_slice(&data_size.to_le_bytes());
    body
}

pub fn update_led_body(led: u16, rgb: [u8; 3]) -> Vec<u8> {
    let mut body = Vec::with_capacity(8);
    body.extend_from_slice(&i32::from(led).to_le_bytes());
    body.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0]);
    body
}

pub fn paints_color(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    !(n.contains("rainbow")
        || n.contains("cycle")
        || n.contains("wave")
        || n.contains("spectrum")
        || n.contains("random")
        || n == "off")
}

fn write_mode(
    stream: &mut TcpStream,
    idx: u32,
    proto: u32,
    data: &[u8],
    label: &str,
    rgb: [u8; 3],
) -> Result<(), String> {
    let (mode_idx, mut blob) = find_mode_at(data, label, proto)?;
    if paints_color(label) {
        tune_mode(&mut blob, proto, rgb);
    } else {
        fill_motion(&mut blob, proto);
    }
    write_pkt(
        stream,
        idx,
        UPDATE_MODE,
        &update_mode_body(mode_idx as i32, &blob),
    )
}

fn find_data(stream: &mut TcpStream, name: &str, proto: u32) -> Result<(u32, Vec<u8>), String> {
    let want = name.trim().to_ascii_lowercase();
    if want.is_empty() {
        return Err("device required".into());
    }
    write_pkt(stream, 0, COUNT, &[])?;
    let payload = crate::openrgb_proto::read_pkt(stream)?;
    if payload.len() < 4 {
        return Err("short count payload".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count bytes")?) as usize;
    if n > 64 {
        return Err("implausible controller count".into());
    }
    let mut found = None;
    for idx in 0..n {
        let data = request_data(stream, idx as u32, proto)?;
        let got = parse_name(&data).unwrap_or_default();
        if got.to_ascii_lowercase() == want {
            found = Some((idx as u32, data));
        }
    }
    found.ok_or_else(|| format!("OpenRGB has no controller named {name}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn rainbow_does_not_paint_mode_colors() {
        assert!(!super::paints_color("Rainbow Wave"));
        assert!(!super::paints_color("Spectrum Cycle"));
        assert!(super::paints_color("Static"));
        assert!(super::paints_color("Breathing"));
    }
}
