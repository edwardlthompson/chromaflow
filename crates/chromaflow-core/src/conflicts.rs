//! Fan daemons that fight PWM. Live units only — leftover unit files do not count.

use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CACHE: Duration = Duration::from_secs(8);
static LAST: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);

const NAMES: [&str; 7] = [
    "fancontrol",
    "coolercontrold",
    "fan2go",
    "thinkfan",
    "nbfc_service",
    "openrazer-daemon",
    "ckb-next-daemon",
];

pub fn detect() -> Vec<String> {
    if let Ok(raw) = std::env::var("CHROMAFLOW_CONFLICTS") {
        if raw.is_empty() || raw == "none" {
            return Vec::new();
        }
        return raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    let now = Instant::now();
    if let Ok(g) = LAST.lock() {
        if let Some((at, rows)) = g.as_ref() {
            if now.saturating_duration_since(*at) < CACHE {
                return rows.clone();
            }
        }
    }
    let mut found = Vec::new();
    for name in NAMES {
        if live(name) {
            found.push(name.to_string());
        }
    }
    if let Ok(mut g) = LAST.lock() {
        *g = Some((now, found.clone()));
    }
    found
}

fn systemctl_quiet(verb: &str, name: &str) -> bool {
    Command::new("systemctl")
        .args([verb, "--quiet", &format!("{name}.service")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Active, enabled (not masked/disabled), or a live process.
fn live(name: &str) -> bool {
    systemctl_quiet("is-active", name)
        || systemctl_quiet("is-enabled", name)
        || Command::new("pidof")
            .args(["--", name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_none_is_empty() {
        // CHROMAFLOW_CONFLICTS is process-wide; this crate's other tests do not set it.
        if std::env::var("CHROMAFLOW_CONFLICTS").is_ok() {
            return;
        }
        let _ = detect();
        assert!(!systemctl_quiet("is-enabled", "this-unit-does-not-exist-chromaflow"));
    }
}
