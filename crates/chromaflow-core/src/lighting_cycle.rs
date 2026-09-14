//! Host Cycle All: 16-bit hue, Direct Keychron fill, live mouse HID, live GPU I2C. No PWM.

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::keychron_apply;

static STOP: AtomicBool = AtomicBool::new(true);
static SPEED: AtomicU8 = AtomicU8::new(128);
static PHASE: AtomicU16 = AtomicU16::new(0);
static JOIN: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

pub const GPU_MS: u64 = 500;

pub fn hue_at(ms: u64, speed: u8) -> u8 {
    let s = u64::from((u16::from(speed.max(1)) / 4).max(1));
    ((ms.saturating_mul(s)) / 256) as u8
}

pub fn period_ms(speed: u8) -> u64 {
    (2_500_000 / u64::from(speed.max(8))).clamp(16_000, 40_000)
}

pub fn rgb16(h: u16) -> [u8; 3] {
    let x = u32::from(h) * 6;
    let region = (x / 65536) as u8;
    let t = ((x % 65536) * 255 / 65535) as u8;
    let q = 255u8.saturating_sub(t);
    match region {
        0 => [255, t, 0],
        1 => [q, 255, 0],
        2 => [0, 255, t],
        3 => [0, q, 255],
        4 => [t, 0, 255],
        _ => [255, 0, q],
    }
}

pub fn running() -> bool {
    !STOP.load(Ordering::SeqCst)
}

pub fn current_hue() -> u8 {
    (PHASE.load(Ordering::Relaxed) >> 8) as u8
}

pub fn set_running(on: bool, speed: u8) {
    SPEED.store(speed.max(1), Ordering::Relaxed);
    if !on {
        stop();
        return;
    }
    if STOP
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    match thread::Builder::new().name("cf-cycle".into()).spawn(run) {
        Ok(h) => store(Some(h)),
        Err(_) => STOP.store(true, Ordering::SeqCst),
    }
}

fn store(h: Option<thread::JoinHandle<()>>) {
    match JOIN.lock() {
        Ok(mut g) => *g = h,
        Err(p) => *p.into_inner() = h,
    }
}

fn stop() {
    STOP.store(true, Ordering::SeqCst);
    let h = match JOIN.lock() {
        Ok(mut g) => g.take(),
        Err(p) => p.into_inner().take(),
    };
    if let Some(h) = h {
        let _ = h.join();
    }
}

fn nap_until_stop(ms: u64) {
    let mut left = ms;
    while left > 0 && !STOP.load(Ordering::Relaxed) {
        let step = left.min(50);
        thread::sleep(Duration::from_millis(step));
        left -= step;
    }
}

fn nap(t0: Instant, ms: u64) {
    let wait = Duration::from_millis(ms);
    if let Some(left) = wait.checked_sub(t0.elapsed()) {
        thread::sleep(left);
    }
}

fn phase_at(t0: Instant, speed: u8) -> u16 {
    let period = period_ms(speed).max(1);
    let ms = t0.elapsed().as_millis() as u64;
    ((ms % period) * 65536 / period) as u16
}

fn run() {
    let t0 = Instant::now();
    let lamps = thread::Builder::new()
        .name("cf-lamps".into())
        .spawn(run_lamps);
    let gpu = thread::Builder::new().name("cf-gpu".into()).spawn(run_gpu);
    while !STOP.load(Ordering::Relaxed) {
        let p = phase_at(t0, SPEED.load(Ordering::Relaxed));
        PHASE.store(p, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(16));
    }
    if let Ok(h) = lamps {
        let _ = h.join();
    }
    if let Ok(h) = gpu {
        let _ = h.join();
    }
    let _ = crate::prime_apply::save();
}

fn run_gpu() {
    let mut last = None;
    while !STOP.load(Ordering::Relaxed) {
        let t0 = Instant::now();
        let rgb = rgb16(PHASE.load(Ordering::Relaxed));
        if last != Some(rgb) {
            if crate::gpu_apply::set_snap(rgb).is_ok() {
                last = Some(rgb);
            } else {
                nap_until_stop(5_000);
                continue;
            }
        }
        let used = t0.elapsed().as_millis() as u64;
        nap_until_stop(GPU_MS.saturating_sub(used));
    }
}

fn run_lamps() {
    let mut n = 0u32;
    let mut last = None;
    while !STOP.load(Ordering::Relaxed) {
        let t0 = Instant::now();
        let rgb = rgb16(PHASE.load(Ordering::Relaxed));
        let hsv = keychron_apply::rgb_to_hsv(rgb);
        if last != Some(hsv) {
            last = Some(hsv);
            let _ = keychron_apply::set_fill(rgb);
        }
        let hex = format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        let _ = crate::arena_apply::set_color(rgb);
        let _ = crate::prime_apply::set_live(rgb);
        let kind = if n % 12 == 0 { "uniform" } else { "soft" };
        let _ = crate::fusion_hid::set(kind, &hex);
        n = n.wrapping_add(1);
        nap(t0, 120);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hue_walks() {
        assert_eq!(super::hue_at(0, 128), 0);
        assert_eq!(super::rgb16(0), [255, 0, 0]);
        assert_ne!(super::rgb16(0), super::rgb16(4000));
        assert!(super::period_ms(128) >= 16_000);
        assert!(super::GPU_MS == 500);
        assert!(!super::running());
    }
}
