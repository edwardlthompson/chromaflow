//! Paint every native lamp from one hex, then retry misses. No PWM. No OpenRGB C++.

use std::sync::{Mutex, MutexGuard};
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

fn keychron(host: bool, rgb: [u8; 3], hex: &str) -> Result<String, String> {
    if host {
        crate::keychron_apply::set_hex_colors(&[format!("#{hex}")])
    } else {
        crate::keychron_apply::set_color(rgb)
    }
}

fn push(host: bool, rgb: [u8; 3], hex: &str) -> Result<String, String> {
    crate::keychron_preview::drop_session();
    let (mut notes, mut errs) = thread::scope(|s| {
        let k = s.spawn(|| keychron(host, rgb, hex));
        let a = s.spawn(|| crate::arena_apply::set_color(rgb));
        let p = s.spawn(|| crate::prime_apply::set_color(rgb));
        let g = s.spawn(|| crate::gpu_apply::set_color(rgb));
        let f = s.spawn(|| crate::liquidctl_apply::set_device("sync", hex, "sync"));
        let mut notes = Vec::new();
        let mut errs = Vec::new();
        take("keychron", k.join(), &mut notes, &mut errs);
        take("arena", a.join(), &mut notes, &mut errs);
        take("prime", p.join(), &mut notes, &mut errs);
        take("gpu", g.join(), &mut notes, &mut errs);
        take("fusion", f.join(), &mut notes, &mut errs);
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
        Ok(format!("broadcast {hex}"))
    } else {
        Ok(format!("broadcast {hex} ({})", errs.join("; ")))
    }
}

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
