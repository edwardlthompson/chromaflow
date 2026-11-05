//! Competing fan / RGB controllers. Live processes only — leftovers do not count.
//! Entries are friendly product names for the Cooling banner and Support list.

#[cfg(not(windows))]
use std::process::Command;
#[cfg(not(windows))]
use std::sync::Mutex;
#[cfg(not(windows))]
use std::time::{Duration, Instant};

#[cfg(not(windows))]
const CACHE: Duration = Duration::from_secs(8);
#[cfg(not(windows))]
static LAST: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);

/// Linux unit / process name → display name for uninstall guidance.
#[cfg(not(windows))]
const CATALOG: &[(&str, &str)] = &[
    ("fancontrol", "fancontrol"),
    ("coolercontrold", "CoolerControl"),
    ("coolercontrol", "CoolerControl"),
    ("fan2go", "fan2go"),
    ("thinkfan", "thinkfan"),
    ("nbfc_service", "NoteBook FanControl"),
    ("nbfc", "NoteBook FanControl"),
    ("openrazer-daemon", "OpenRazer"),
    ("ckb-next-daemon", "ckb-next"),
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
    #[cfg(windows)]
    {
        return crate::windows::conflicts::detect();
    }
    #[cfg(not(windows))]
    {
        let now = Instant::now();
        if let Ok(g) = LAST.lock() {
            if let Some((at, rows)) = g.as_ref() {
                if now.saturating_duration_since(*at) < CACHE {
                    return rows.clone();
                }
            }
        }
        let mut found = Vec::new();
        for (proc, display) in CATALOG {
            if live(proc) && !found.iter().any(|f| f == display) {
                found.push((*display).to_string());
            }
        }
        found.sort();
        if let Ok(mut g) = LAST.lock() {
            *g = Some((now, found.clone()));
        }
        found
    }
}

#[cfg(not(windows))]
fn systemctl_quiet(verb: &str, name: &str) -> bool {
    Command::new("systemctl")
        .args([verb, "--quiet", &format!("{name}.service")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Active, enabled (not masked/disabled), or a live process.
#[cfg(not(windows))]
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
        if std::env::var("CHROMAFLOW_CONFLICTS").is_ok() {
            return;
        }
        let _ = detect();
        #[cfg(not(windows))]
        assert!(!systemctl_quiet(
            "is-enabled",
            "this-unit-does-not-exist-chromaflow"
        ));
    }

    #[test]
    fn env_lists_display_names() {
        std::env::set_var("CHROMAFLOW_CONFLICTS", "CoolerControl,fancontrol");
        assert_eq!(
            detect(),
            vec!["CoolerControl".to_string(), "fancontrol".to_string()]
        );
        std::env::set_var("CHROMAFLOW_CONFLICTS", "none");
        assert!(detect().is_empty());
        std::env::remove_var("CHROMAFLOW_CONFLICTS");
    }
}
