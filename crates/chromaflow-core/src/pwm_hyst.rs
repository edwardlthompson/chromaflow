//! Last-duty hysteresis and step clamp. Never writes sysfs.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Last {
    pct: u8,
    temp: f32,
    at: Instant,
}

static LAST: Mutex<Option<HashMap<String, Last>>> = Mutex::new(None);
static PAUSE: Mutex<bool> = Mutex::new(false);

pub fn paused() -> bool {
    *PAUSE.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_paused(v: bool) {
    if let Ok(mut g) = PAUSE.lock() {
        *g = v;
    }
}

pub fn key(dir: &str, pwm: &str) -> String {
    format!("{dir}/{pwm}")
}

pub fn clamp(
    id: &str,
    target: u8,
    temp: f32,
    hyst_c: u8,
    step_up: u8,
    step_down: u8,
    response_ms: u32,
) -> u8 {
    let now = Instant::now();
    let mut slot = LAST.lock().unwrap_or_else(|e| e.into_inner());
    let map = slot.get_or_insert_with(HashMap::new);
    let Some(prev) = map.get(id) else {
        map.insert(
            id.to_string(),
            Last {
                pct: target,
                temp,
                at: now,
            },
        );
        return target;
    };
    if response_ms > 0 && now.saturating_duration_since(prev.at) < Duration::from_millis(u64::from(response_ms))
    {
        return prev.pct;
    }
    if hyst_c > 0 && (temp - prev.temp).abs() < f32::from(hyst_c) && target != prev.pct {
        return prev.pct;
    }
    let mut next = target;
    if step_up > 0 && next > prev.pct {
        next = prev.pct.saturating_add(step_up).min(next);
    }
    if step_down > 0 && next < prev.pct {
        next = prev.pct.saturating_sub(step_down).max(next);
    }
    map.insert(
        id.to_string(),
        Last {
            pct: next,
            temp,
            at: now,
        },
    );
    next
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_tick_is_target() {
        super::set_paused(false);
        assert!(!super::paused());
        assert_eq!(super::clamp("t/pwm1", 40, 50.0, 3, 5, 2, 0), 40);
    }
}
