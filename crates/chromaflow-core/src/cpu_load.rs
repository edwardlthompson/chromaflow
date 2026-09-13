//! CPU busy ratio from `/proc/stat`. Gauges only. No PWM.

use std::fs;
use std::sync::Mutex;

static PREV: Mutex<Option<(u64, u64)>> = Mutex::new(None);

/// Returns `(total, idle+iowait)` jiffies from the aggregate `cpu ` line.
pub fn parse_proc_stat(text: &str) -> Option<(u64, u64)> {
    let line = text.lines().find(|l| l.starts_with("cpu "))?;
    let mut idle = 0u64;
    let mut total = 0u64;
    for (i, part) in line.split_whitespace().skip(1).enumerate() {
        let v: u64 = part.parse().ok()?;
        total = total.saturating_add(v);
        if i == 3 || i == 4 {
            idle = idle.saturating_add(v);
        }
    }
    if total == 0 {
        return None;
    }
    Some((total, idle))
}

pub fn cached_cpu_load() -> Option<f64> {
    let next = fs::read_to_string("/proc/stat")
        .ok()
        .as_deref()
        .and_then(parse_proc_stat)?;
    let mut slot = PREV.lock().ok()?;
    let prev = *slot;
    *slot = Some(next);
    let (pt, pi) = prev?;
    let dt = next.0.saturating_sub(pt);
    let di = next.1.saturating_sub(pi);
    if dt == 0 {
        return None;
    }
    Some((1.0 - (di as f64 / dt as f64)).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_stat_idle_and_busy() {
        let a = parse_proc_stat("cpu  10 0 10 80 0 0 0 0\ncpu0 1 0 1 8\n").unwrap();
        assert_eq!(a, (100, 80));
        let b = parse_proc_stat("cpu  20 0 20 80 0 0 0 0\n").unwrap();
        assert_eq!(b, (120, 80));
        assert!(parse_proc_stat("").is_none());
    }
}
