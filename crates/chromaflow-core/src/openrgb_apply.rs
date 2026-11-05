//! SET_CUSTOM_MODE + UPDATE_LEDS on localhost. No hidraw, no PWM.

use std::net::TcpStream;

use crate::openrgb_parse::{find_mode_at, parse_led_count_at, tune_mode};
use crate::openrgb_proto::{
    close, open_client, parse_name, write_pkt, COUNT, SET_CUSTOM, UPDATE_LEDS, UPDATE_MODE,
};

pub fn set_color(name: &str, rgb: [u8; 3]) -> Result<String, String> {
    let want = name.trim().to_ascii_lowercase();
    if want.is_empty() {
        return Err("device required".into());
    }
    let _sdk = crate::openrgb_proto::lock_sdk();
    let (mut stream, proto) = open_client()?;
    write_pkt(&mut stream, 0, COUNT, &[])?;
    let payload = crate::openrgb_proto::read_pkt(&mut stream)?;
    if payload.len() < 4 {
        return Err("short count payload".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count bytes")?) as usize;
    if n > 64 {
        return Err("implausible controller count".into());
    }
    let mut exact: Option<(u32, Vec<u8>)> = None;
    let mut fuzzy: Option<(u32, Vec<u8>)> = None;
    for idx in 0..n {
        let data = crate::openrgb_proto::request_data(&mut stream, idx as u32, proto)?;
        let got = parse_name(&data).unwrap_or_default();
        let gl = got.to_ascii_lowercase();
        if gl == want {
            exact = Some((idx as u32, data));
            break;
        }
        if fuzzy.is_none() && name_matches(&want, &gl) {
            fuzzy = Some((idx as u32, data));
        }
    }
    let Some((idx, data)) = exact.or(fuzzy) else {
        close(&mut stream);
        return Err(format!("OpenRGB has no controller named {name}"));
    };
    let leds = parse_led_count_at(&data, proto)?;
    if leds == 0 || leds > 2048 {
        close(&mut stream);
        return Err("implausible LED count".into());
    }
    // Leave Breathing/etc.: force Direct/Custom then paint LEDs (MSI GPU especially).
    force_direct_fill(&mut stream, idx, proto, &data, leds, rgb)?;
    let dled = crate::openrgb_dled::paint(&mut stream, idx, name, rgb).unwrap_or(0);
    std::thread::sleep(std::time::Duration::from_millis(40));
    close(&mut stream);
    if dled > 0 {
        Ok(format!(
            "set {name} ({leds} LEDs + {dled} D_LED zones) via OpenRGB SDK"
        ))
    } else {
        Ok(format!("set {name} ({leds} LEDs) via OpenRGB SDK"))
    }
}

/// Switch controller out of Breathing/Static firmware modes into Direct, then UPDATE_LEDS.
pub fn force_direct_fill(
    stream: &mut TcpStream,
    idx: u32,
    proto: u32,
    data: &[u8],
    leds: u16,
    rgb: [u8; 3],
) -> Result<(), String> {
    for label in ["Direct", "direct", "Custom", "Static"] {
        if let Ok((mode_idx, mut blob)) = find_mode_at(data, label, proto) {
            if label.eq_ignore_ascii_case("static") {
                tune_mode(&mut blob, proto, rgb);
            }
            let _ = write_pkt(
                stream,
                idx,
                UPDATE_MODE,
                &crate::openrgb_mode::update_mode_body(mode_idx as i32, &blob),
            );
            break;
        }
    }
    write_pkt(stream, idx, SET_CUSTOM, &[])?;
    write_pkt(stream, idx, UPDATE_LEDS, &update_leds_body(leds, rgb))?;
    Ok(())
}

fn name_matches(want: &str, got: &str) -> bool {
    if want.is_empty() || got.is_empty() {
        return false;
    }
    if got.contains(want) || want.contains(got) {
        return true;
    }
    let pairs = [
        ("keychron", "keychron"),
        ("q6", "keychron"),
        ("arena", "arena"),
        ("prime", "prime"),
        ("aerox", "aerox"),
        ("rival", "rival"),
        ("steelseries", "steelseries"),
        ("aorus", "aorus"),
        ("4090", "4090"),
        ("suprim", "suprim"),
    ];
    for (a, b) in pairs {
        if want.contains(a) && got.contains(b) {
            return true;
        }
    }
    false
}

/// Body must equal header `pkt_size`. OpenRGB git2478 rejects UPDATE_LEDS unless
/// the first u32 equals that full payload (`4 + 2 + 4*n`), not `2 + 4*n`.
pub fn update_leds_body(n: u16, rgb: [u8; 3]) -> Vec<u8> {
    update_leds_colors(&vec![rgb; usize::from(n)])
}

pub fn update_leds_colors(colors: &[[u8; 3]]) -> Vec<u8> {
    let n = u16::try_from(colors.len()).unwrap_or(0);
    let mut body = Vec::with_capacity(6 + 4 * colors.len());
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&n.to_le_bytes());
    for rgb in colors {
        body.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0]);
    }
    let data_size = u32::try_from(body.len()).unwrap_or(0);
    body[0..4].copy_from_slice(&data_size.to_le_bytes());
    body
}
