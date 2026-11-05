//! Paint every native lamp from one hex, then retry misses. No PWM. No OpenRGB C++.

use std::sync::{Mutex, MutexGuard};
#[cfg(not(windows))]
use std::thread;

use crate::lighting_apply::parse_rrggbb;

static PAINT: Mutex<()> = Mutex::new(());

pub fn lock() -> MutexGuard<'static, ()> {
    PAINT.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn with_paint<T>(f: impl FnOnce() -> T) -> T {
    let _g = lock();
    f()
}

fn host_tick(mode: &str) -> bool {
    let m = mode.to_ascii_lowercase();
    m.contains("gauge") || m.contains("breath") || m.contains("flash")
}

pub fn broadcast(color: &str, mode: &str) -> Result<String, String> {
    let rgb = parse_rrggbb(color)?;
    let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
    with_paint(|| push(host_tick(mode), rgb, &hex))
}

#[cfg(not(windows))]
fn keychron(host: bool, rgb: [u8; 3], hex: &str) -> Result<String, String> {
    if host {
        crate::keychron_apply::set_hex_colors(&[format!("#{hex}")])
    } else {
        crate::keychron_apply::set_color(rgb)
    }
}

fn push(host: bool, rgb: [u8; 3], hex: &str) -> Result<String, String> {
    crate::keychron_preview::drop_session();
    #[cfg(windows)]
    {
        let _ = host;
        if crate::openrgb_proto::connect().is_err()
            || crate::openrgb_proto::controller_count().unwrap_or(0) == 0
        {
            let _ = crate::openrgb_spawn::ensure_local_server();
        }
        let _ = crate::openrgb_proto::wait_for_controllers(std::time::Duration::from_secs(15));
        let mut notes = Vec::new();
        let mut errs = Vec::new();
        match openrgb_all(rgb) {
            Ok(msg) => notes.push(msg),
            Err(e) => errs.push(format!("openrgb: {e}")),
        }
        match crate::arena_apply::set_color(rgb) {
            Ok(msg) => notes.push(msg),
            Err(e) => errs.push(format!("arena: {e}")),
        }
        match crate::prime_apply::set_color(rgb) {
            Ok(msg) => notes.push(msg),
            Err(e) => errs.push(format!("prime: {e}")),
        }
        if notes.is_empty() {
            return Err(if errs.is_empty() {
                "Lighting engine not reachable on 127.0.0.1:6742".into()
            } else {
                errs.join("; ")
            });
        }
        if errs.is_empty() {
            Ok(format!("broadcast {hex} — {}", notes.join("; ")))
        } else {
            Ok(format!(
                "broadcast {hex} — {}; misses: {}",
                notes.join("; "),
                errs.join("; ")
            ))
        }
    }
    #[cfg(not(windows))]
    {
        let (mut notes, mut errs) = thread::scope(|s| {
            let k = s.spawn(|| keychron(host, rgb, hex));
            let a = s.spawn(|| crate::arena_apply::set_color(rgb));
            let p = s.spawn(|| crate::prime_apply::set_color(rgb));
            let g = s.spawn(|| crate::gpu_apply::set_color(rgb));
            let f = s.spawn(|| crate::liquidctl_apply::set_device("sync", hex, "sync"));
            let o = s.spawn(|| openrgb_all(rgb));
            let mut notes = Vec::new();
            let mut errs = Vec::new();
            take("keychron", k.join(), &mut notes, &mut errs);
            take("arena", a.join(), &mut notes, &mut errs);
            take("prime", p.join(), &mut notes, &mut errs);
            take("gpu", g.join(), &mut notes, &mut errs);
            take("fusion", f.join(), &mut notes, &mut errs);
            take("openrgb", o.join(), &mut notes, &mut errs);
            (notes, errs)
        });
        if errs.iter().any(|e| e.starts_with("keychron:")) {
            match keychron(host, rgb, hex) {
                Ok(m) => notes.push(m),
                Err(e) => errs.push(format!("keychron-retry: {e}")),
            }
        }
        if notes.is_empty() {
            return Err(errs.join("; "));
        }
        if errs.is_empty() {
            Ok(format!("broadcast {hex} — {}", notes.join("; ")))
        } else {
            Ok(format!(
                "broadcast {hex} — {}; misses: {}",
                notes.join("; "),
                errs.join("; ")
            ))
        }
    }
}

/// Paint every controller on a live localhost OpenRGB SDK (no spawn required).
fn openrgb_all(rgb: [u8; 3]) -> Result<String, String> {
    use crate::openrgb_proto::{close, open_client, parse_name, read_pkt, request_data, write_pkt, COUNT};
    let _sdk = crate::openrgb_proto::lock_sdk();
    let (mut stream, proto) = open_client()?;
    write_pkt(&mut stream, 0, COUNT, &[])?;
    let payload = read_pkt(&mut stream)?;
    if payload.len() < 4 {
        close(&mut stream);
        return Err("short count".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count")?) as usize;
    if n == 0 || n > 64 {
        close(&mut stream);
        return Err("no OpenRGB controllers".into());
    }
    let mut painted = 0u32;
    for idx in 0..n {
        let data = match request_data(&mut stream, idx as u32, proto) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let name = parse_name(&data).unwrap_or_else(|| format!("#{idx}"));
        let leds = crate::openrgb_parse::parse_led_count_at(&data, proto).unwrap_or(0);
        if leds == 0 || leds > 2048 {
            continue;
        }
        if crate::openrgb_apply::force_direct_fill(
            &mut stream,
            idx as u32,
            proto,
            &data,
            leds,
            rgb,
        )
        .is_ok()
        {
            let _ = crate::openrgb_dled::paint(&mut stream, idx as u32, &name, rgb);
            painted += 1;
        }
    }
    close(&mut stream);
    if painted == 0 {
        Err("OpenRGB listed devices but none accepted UPDATE_LEDS".into())
    } else {
        Ok(format!("openrgb painted {painted} controller(s)"))
    }
}

#[cfg(not(windows))]
fn take(
    name: &str,
    joined: thread::Result<Result<String, String>>,
    notes: &mut Vec<String>,
    errs: &mut Vec<String>,
) {
    match joined.unwrap_or(Err("join".into())) {
        Ok(m) => notes.push(m),
        Err(e) => errs.push(format!("{name}: {e}")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_empty() {
        assert!(super::broadcast("", "Solid Color").is_err());
        assert!(super::broadcast("00", "Cycle All").is_err());
        assert!(!super::host_tick("Cycle All"));
        assert!(!super::host_tick("Solid Color"));
        assert!(!super::host_tick("Direct"));
        assert!(super::host_tick("Breathing"));
    }

    #[test]
    fn live_broadcast_polls_leds() {
        if std::env::var("CHROMAFLOW_LIVE_RGB").ok().as_deref() != Some("1") {
            return;
        }
        let red = super::broadcast("FF0000", "Solid Color").expect("solid red");
        assert!(red.contains("FF0000"), "{red}");
        let fx = crate::keychron_apply::poll_effect();
        assert_eq!(fx, Some(1), "SOLID effect {fx:?} after {red}");
    }
}
