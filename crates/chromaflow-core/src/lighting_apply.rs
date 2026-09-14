//! Dispatch Lighting color apply. Native HID, liquidctl Fusion, optional OpenRGB.

pub fn parse_rrggbb(raw: &str) -> Result<[u8; 3], String> {
    let s = raw.trim().trim_start_matches('#');
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("color must be RRGGBB".into());
    }
    let n = u32::from_str_radix(s, 16).map_err(|_| "color")?;
    Ok([
        ((n >> 16) & 0xff) as u8,
        ((n >> 8) & 0xff) as u8,
        (n & 0xff) as u8,
    ])
}

fn ready(backend: &str) {
    if backend == "openrgb" {
        crate::openrgb_spawn::ensure_sdk();
        crate::openrgb_preview::drop_session();
        crate::keychron_preview::drop_session();
    }
    if backend == "keychron" {
        crate::keychron_preview::drop_session();
    }
}

pub fn apply(backend: &str, device: &str, color: &str) -> Result<String, String> {
    crate::lighting_broadcast::with_paint(|| apply_unlocked(backend, device, color))
}

fn apply_unlocked(backend: &str, device: &str, color: &str) -> Result<String, String> {
    if color.trim().is_empty() {
        return Err("color must be RRGGBB".into());
    }
    let backend = crate::lighting_port::rewrite(backend, device);
    ready(&backend);
    let rgb = parse_rrggbb(color)?;
    let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
    match backend.as_str() {
        "openrgb" => crate::openrgb_apply::set_color(device, rgb),
        "arena" => crate::arena_apply::set_color(rgb),
        "prime" => crate::prime_apply::set_color(rgb),
        "liquidctl" => crate::liquidctl_apply::set_device(device, &hex, "fixed"),
        "keychron" => crate::keychron_apply::set_color(rgb),
        "msi_gpu" => crate::gpu_apply::set_color(rgb),
        _ => Err("unknown lighting backend".into()),
    }
}

pub fn apply_mode(backend: &str, device: &str, mode: &str, color: &str) -> Result<String, String> {
    crate::lighting_broadcast::with_paint(|| apply_mode_unlocked(backend, device, mode, color))
}

fn apply_mode_unlocked(
    backend: &str,
    device: &str,
    mode: &str,
    color: &str,
) -> Result<String, String> {
    let backend = crate::lighting_port::rewrite(backend, device);
    ready(&backend);
    if backend == "liquidctl" {
        let hex = parse_rrggbb(color)
            .map(|rgb| format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]))
            .unwrap_or_else(|_| "FFFFFF".into());
        return crate::liquidctl_apply::set_device(device, &hex, mode);
    }
    let rgb = parse_rrggbb(color)?;
    if backend == "msi_gpu" {
        return crate::gpu_apply::set_color(rgb);
    }
    if backend != "openrgb" {
        return Err("effects need OpenRGB SDK".into());
    }
    crate::openrgb_mode::set_mode(device, mode, rgb)
}

pub fn apply_led(backend: &str, device: &str, led: u16, color: &str) -> Result<String, String> {
    crate::lighting_broadcast::with_paint(|| apply_led_unlocked(backend, device, led, color))
}

fn apply_led_unlocked(
    backend: &str,
    device: &str,
    led: u16,
    color: &str,
) -> Result<String, String> {
    if color.trim().is_empty() {
        return Err("color must be RRGGBB".into());
    }
    let backend = crate::lighting_port::rewrite(backend, device);
    ready(&backend);
    let rgb = parse_rrggbb(color)?;
    if backend == "keychron" {
        return crate::keychron_apply::set_led(led, rgb);
    }
    if backend == "liquidctl" {
        return crate::liquidctl_apply::set_led(
            device,
            led,
            &format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
        );
    }
    if backend != "openrgb" {
        return Err("per-LED needs OpenRGB SDK".into());
    }
    crate::openrgb_mode::set_led(device, led, rgb)
}

#[cfg(test)]
mod tests {
    use super::{apply, apply_led, apply_mode, parse_rrggbb};

    #[test]
    fn hex_and_unknown_backend() {
        assert_eq!(parse_rrggbb("#0052ff").unwrap(), [0, 0x52, 0xff]);
        assert_eq!(parse_rrggbb("aabbcc").unwrap(), [0xaa, 0xbb, 0xcc]);
        assert!(parse_rrggbb("").is_err());
        assert!(parse_rrggbb("xyzxyz").is_err());
        assert!(apply("hidraw", "Arena", "0052FF").is_err());
        assert!(apply("openrgb", "", "0052FF").is_err());
        assert!(apply("keychron", "Q6", "").is_err());
        let zone = crate::openrgb_dled::zone_body(0, 2, [0, 0x52, 0xff]);
        assert_eq!(zone.len(), 18);
        assert_eq!(&zone[0..4], &(zone.len() as u32).to_le_bytes());
        let arena = crate::arena_apply::color_report([0, 0xe5, 0xff]);
        assert_eq!(arena[0], 0x06);
        let prime = crate::prime_apply::color_report([0, 0x52, 0xff]);
        assert_eq!(prime[1], 0x62);
        assert_eq!(crate::prime_apply::save_report()[1], 0x59);
        let body = crate::openrgb_apply::update_leds_body(2, [0, 0x52, 0xff]);
        assert_eq!(body.len(), 14);
        assert_eq!(&body[0..4], &(body.len() as u32).to_le_bytes());
        assert_eq!(&body[4..6], &2u16.to_le_bytes());
        assert_eq!(&body[6..10], &[0, 0x52, 0xff, 0]);
        let mode = crate::openrgb_mode::update_mode_body(1, &[1, 2, 3]);
        assert_eq!(mode.len(), 11);
        assert_eq!(&mode[0..4], &(mode.len() as u32).to_le_bytes());
        let led = crate::openrgb_mode::update_led_body(3, [0, 0x52, 0xff]);
        assert_eq!(led.len(), 8);
        assert_eq!(&led[0..4], &3i32.to_le_bytes());
        assert!(apply_mode("arena", "Arena", "Breathing", "0052FF").is_err());
        assert!(apply_led("arena", "Arena", 0, "0052FF").is_err());
        let mut data = vec![0u8; 8];
        for s in [b"X\0".as_slice(), b"\0", b"\0", b"\0", b"\0"] {
            data.extend_from_slice(&(s.len() as u16).to_le_bytes());
            data.extend_from_slice(s);
        }
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&3u16.to_le_bytes());
        assert_eq!(crate::openrgb_parse::parse_led_count(&data).unwrap(), 3);
    }
}
