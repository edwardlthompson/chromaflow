//! Windows apps that fight SuperIO PWM or exclusive USB RGB.
//! Returns friendly product names so users know what to uninstall.

use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CACHE: Duration = Duration::from_secs(8);
const INSTALL_CACHE: Duration = Duration::from_secs(60);
static LAST: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);
static LAST_INSTALL: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);

/// Process image stems (no .exe) → display name for the conflict banner.
const CATALOG: &[(&[&str], &str)] = &[
    (&["FanControl"], "Fan Control"),
    (&["LibreHardwareMonitor", "LibreHardwareMonitor.UI"], "Libre Hardware Monitor"),
    (&["OpenHardwareMonitor"], "Open Hardware Monitor"),
    (&["ArmouryCrate", "ArmouryCrate.UserSessionHelper", "ArmouryCrate.Service", "AsusSoftwareManager", "LightingService", "AacHaloLight"], "ASUS Armoury Crate"),
    (&["iCUE", "Corsair.Service", "CorsairService"], "Corsair iCUE"),
    (&["CAM", "NZXT CAM", "NZXT-CAM"], "NZXT CAM"),
    (&["SignalRgb", "SignalRGB"], "SignalRGB"),
    (
        &[
            "SteelSeriesGG",
            "SteelSeriesGGEZ",
            "SteelSeriesGGClient",
            "SteelSeriesEngine",
            "SteelSeriesEngine3",
            "SteelSeriesPrism",
            "GGClient",
            "GGWave",
            "SSOverlay",
            "SteelSeriesSonar",
            "SteelSeriesGGUpdateService",
            "SteelSeriesGGUpdateServiceProxy",
            "SteelSeriesCaptureSvc",
        ],
        "SteelSeries GG",
    ),
    (&["RazerCentralService", "Razer Synapse Service", "RazerAppEngine", "GameManagerService"], "Razer Synapse"),
    (&["MSIAfterburner", "Afterburner", "MysticLight"], "MSI Afterburner / Mystic Light"),
    (
        &[
            "RGBFusion",
            "EasyTune",
            "EasyTuneEngineService",
            "GCC",
            "GigabyteControlCenter",
            "AorusGraphicsEngine",
        ],
        "Gigabyte Control Center / EasyTune / RGB Fusion",
    ),
    (&["MasterPlus", "MasterPlusX", "MP_Service"], "Cooler Master MasterPlus"),
    (&["TT RGB Plus", "TT_RGB_Plus"], "Thermaltake TT RGB Plus"),
    (&["L-Connect", "LConnect", "LianLi.LConnect"], "Lian Li L-Connect"),
    (&["SpeedFan"], "SpeedFan"),
    (&["ArgusMonitor"], "Argus Monitor"),
    (&["nbfc", "NoteBookFanControl", "NBFCClient"], "NoteBook FanControl"),
    (&["EVGAPrecisionX1", "PrecisionX_x64"], "EVGA Precision X1"),
];

const INSTALL_HINTS: &[(&str, &str)] = &[
    ("FanControl", "Fan Control"),
    ("LibreHardwareMonitor", "Libre Hardware Monitor"),
    ("ASUS\\Armoury Crate", "ASUS Armoury Crate"),
    ("Corsair\\Corsair iCUE", "Corsair iCUE"),
    ("Corsair\\iCUE", "Corsair iCUE"),
    ("NZXT\\NZXT CAM", "NZXT CAM"),
    ("SignalRgb", "SignalRGB"),
    ("SteelSeries", "SteelSeries GG"),
    ("Razer", "Razer Synapse"),
    ("MSI\\Afterburner", "MSI Afterburner / Mystic Light"),
    ("GIGABYTE\\Control Center", "Gigabyte Control Center / RGB Fusion"),
    ("Cooler Master\\MasterPlus", "Cooler Master MasterPlus"),
    // Standalone OpenRGB only — never the private engine under ChromaFlow\engine.
    ("OpenRGB", "OpenRGB (standalone)"),
];

/// Live competing processes only (inventory banner / PWM gate).
/// Installed-but-quit apps are listed by [`installed`] for Support, not here —
/// otherwise Take control cannot clear the banner while SteelSeries remains on disk.
pub fn detect() -> Vec<String> {
    if let Ok(raw) = std::env::var("CHROMAFLOW_CONFLICTS") {
        if raw.is_empty() || raw == "none" {
            return Vec::new();
        }
        return raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    let now = Instant::now();
    if let Ok(g) = LAST.lock() {
        if let Some((at, rows)) = g.as_ref() {
            if now.saturating_duration_since(*at) < CACHE {
                return rows.clone();
            }
        }
    }
    let mut found = running();
    found.sort();
    found.dedup();
    if let Ok(mut g) = LAST.lock() {
        *g = Some((now, found.clone()));
    }
    found
}

/// Friendly names for apps still installed (Support list); may overlap with [`detect`].
pub fn installed_names() -> Vec<String> {
    let mut rows = installed();
    rows.sort();
    rows.dedup();
    rows
}

fn running() -> Vec<String> {
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FO", "CSV", "/NH"]);
    let Ok(out) = crate::win_cmd::hidden_capture(cmd).output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    let mut found = Vec::new();
    for (images, display) in CATALOG {
        if images.iter().any(|img| {
            let stem = img.to_ascii_lowercase();
            text.contains(&format!("\"{stem}.exe\"")) || text.contains(&format!("{stem}.exe"))
        }) {
            found.push((*display).to_string());
        }
    }
    found
}

fn installed() -> Vec<String> {
    let now = Instant::now();
    if let Ok(g) = LAST_INSTALL.lock() {
        if let Some((at, rows)) = g.as_ref() {
            if now.saturating_duration_since(*at) < INSTALL_CACHE {
                return rows.clone();
            }
        }
    }
    let program_roots = [
        std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into()),
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into()),
    ];
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let mut found = Vec::new();
    for (rel, display) in INSTALL_HINTS {
        // Standalone OpenRGB lives under Program Files only. Do not match
        // %LOCALAPPDATA%\chromaflow\engine\OpenRGB (private engine) or leftover config dirs.
        let roots: Vec<&str> = if *rel == "OpenRGB" {
            program_roots.iter().map(|s| s.as_str()).collect()
        } else {
            program_roots
                .iter()
                .map(|s| s.as_str())
                .chain(std::iter::once(local.as_str()))
                .collect()
        };
        for root in roots {
            if root.is_empty() {
                continue;
            }
            let path = Path::new(root).join(rel);
            let hit = if *rel == "OpenRGB" {
                path.join("OpenRGB.exe").is_file()
            } else {
                path.is_dir()
            };
            if hit {
                found.push((*display).to_string());
                break;
            }
        }
    }
    if let Ok(mut g) = LAST_INSTALL.lock() {
        *g = Some((now, found.clone()));
    }
    found
}

fn clear_detect_cache() {
    if let Ok(mut g) = LAST.lock() {
        *g = None;
    }
    if let Ok(mut g) = LAST_INSTALL.lock() {
        *g = None;
    }
}

/// Kill catalog competitors that are actually running (one tasklist pass).
pub fn kill_running_competitors() -> String {
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FO", "CSV", "/NH"]);
    let text = crate::win_cmd::hidden_capture(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase())
        .unwrap_or_default();
    let mut killed = Vec::new();
    for (images, display) in CATALOG {
        for img in *images {
            let stem = img.to_ascii_lowercase();
            let hit = text.contains(&format!("\"{stem}.exe\"")) || text.contains(&format!("{stem}.exe"));
            if !hit {
                continue;
            }
            let mut kill = Command::new("taskkill");
            kill.args(["/IM", &format!("{img}.exe"), "/F", "/T"]);
            if crate::win_cmd::hidden(kill)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                killed.push(format!("{display} ({img})"));
            }
        }
    }
    let openrgb = crate::openrgb_spawn::kill_non_private_openrgb();
    if killed.is_empty() {
        format!("no catalog processes to kill; {openrgb}")
    } else {
        format!("killed {}; {openrgb}", killed.join(", "))
    }
}

/// Stop Gigabyte EasyTune / Fan Control from reclaiming SuperIO at logon.
/// Services → manual (reversible); scheduled tasks disabled; common Run values cleared.
pub fn suppress_gigabyte_fan_autostart() -> String {
    let mut notes = Vec::new();
    for img in [
        "EasyTune",
        "EasyTuneEngineService",
        "GCC",
        "GigabyteControlCenter",
        "RGBFusion",
        "FanControl",
        "AorusGraphicsEngine",
    ] {
        let mut kill = Command::new("taskkill");
        kill.args(["/IM", &format!("{img}.exe"), "/F", "/T"]);
        let _ = crate::win_cmd::hidden(kill).status();
    }
    for svc in ["EasyTuneEngineService", "FanControl.Service", "GigabyteUpdateService"] {
        let mut stop = Command::new("sc");
        stop.args(["stop", svc]);
        let _ = crate::win_cmd::hidden(stop).status();
        let mut cfg = Command::new("sc");
        cfg.args(["config", svc, "start=", "demand"]);
        if crate::win_cmd::hidden(cfg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            notes.push(format!("service {svc} set manual"));
        }
    }
    for task in ["EasyTune", "EasyTune 1", "FanControl"] {
        let mut cmd = Command::new("schtasks");
        cmd.args(["/Change", "/TN", task, "/DISABLE"]);
        if crate::win_cmd::hidden(cmd)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            notes.push(format!("task {task} disabled"));
        }
    }
    for (hive, label) in [
        (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "HKCU"),
        (r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run", "HKLM"),
    ] {
        for val in ["EasyTune", "FanControl", "GCC", "RGBFusion", "GigabyteControlCenter"] {
            let mut cmd = Command::new("reg");
            cmd.args(["delete", hive, "/v", val, "/f"]);
            if crate::win_cmd::hidden(cmd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                notes.push(format!("cleared {label} Run\\{val}"));
            }
        }
    }
    if notes.is_empty() {
        "Gigabyte/Fan Control autostart: no service/task/Run changes".into()
    } else {
        notes.join("; ")
    }
}

/// Stop SteelSeries GG from coming back: strip Run keys + stop updater (fast, no huge PS scan).
pub fn suppress_steelseries_autostart() -> String {
    let mut notes = Vec::new();
    for img in [
        "SteelSeriesGGEZ",
        "SteelSeriesGG",
        "SteelSeriesGGClient",
        "SteelSeriesPrism",
        "GGClient",
        "GGWave",
    ] {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/IM", &format!("{img}.exe"), "/F", "/T"]);
        let _ = crate::win_cmd::hidden(cmd).status();
    }
    for (hive, label) in [
        (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "HKCU"),
        (r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run", "HKLM"),
    ] {
        let mut cmd = Command::new("reg");
        cmd.args(["delete", hive, "/v", "SteelSeriesGG", "/f"]);
        if crate::win_cmd::hidden(cmd)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            notes.push(format!("cleared {label} Run\\SteelSeriesGG"));
        }
    }
    for svc in [
        "SteelSeriesGGUpdateServiceProxy",
        "SteelSeriesGGUpdateService",
    ] {
        let mut stop = Command::new("sc");
        stop.args(["stop", svc]);
        let _ = crate::win_cmd::hidden(stop).status();
        let mut cfg = Command::new("sc");
        cfg.args(["config", svc, "start=", "demand"]);
        if crate::win_cmd::hidden(cfg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            notes.push(format!("service {svc} set manual"));
        }
    }
    if notes.is_empty() {
        "SteelSeries autostart: no Run key / service changes".into()
    } else {
        notes.join("; ")
    }
}

/// User-requested take-over: quit lockers, stop GG relaunch, clear standalone OpenRGB.
pub fn seize_control() -> serde_json::Value {
    let before = detect();
    let mut notes = Vec::new();
    notes.push(kill_running_competitors());
    notes.push(suppress_gigabyte_fan_autostart());
    notes.push(suppress_steelseries_autostart());
    notes.push(crate::windows::hid::release_steelseries_locks());
    notes.push(uninstall_standalone_openrgb());
    clear_detect_cache();
    let running_now = running();
    let after = detect();
    let errors: Vec<String> = running_now
        .iter()
        .map(|n| format!("{n} still running — quit it or remove its startup entry"))
        .collect();
    serde_json::json!({
        "ok": errors.is_empty(),
        "os": "windows",
        "before": before,
        "after": after,
        "detail": notes.join(" · "),
        "errors": errors,
    })
}

/// Try to remove standalone OpenRGB (Program Files). Never touches ChromaFlow\\engine.
/// Winget is hard-capped at 30s; always kill non-private OpenRGB.exe afterward.
pub fn uninstall_standalone_openrgb() -> String {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
    let standalone = [
        Path::new(&pf).join("OpenRGB").join("OpenRGB.exe"),
        Path::new(&pf86).join("OpenRGB").join("OpenRGB.exe"),
    ];
    let present = standalone.iter().any(|p| p.is_file());
    let mut notes = Vec::new();
    if present {
        let mut cmd = Command::new("winget");
        cmd.args([
            "uninstall",
            "--id",
            "OpenRGB.OpenRGB",
            "-e",
            "--silent",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]);
        let mut child = match crate::win_cmd::hidden(cmd).spawn() {
            Ok(c) => c,
            Err(e) => {
                notes.push(format!("winget unavailable ({e})"));
                notes.push(crate::openrgb_spawn::kill_non_private_openrgb());
                notes.push("Uninstall OpenRGB from Settings → Apps if it remains".into());
                return notes.join("; ");
            }
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    notes.push(format!("winget uninstall exit {status}"));
                    break;
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(200));
                }
                _ => {
                    let _ = child.kill();
                    notes.push("winget uninstall timed out (30s)".into());
                    break;
                }
            }
        }
    } else {
        notes.push("no Program Files\\OpenRGB install".into());
    }
    notes.push(crate::openrgb_spawn::kill_non_private_openrgb());
    if standalone.iter().any(|p| p.is_file()) {
        notes.push("Uninstall OpenRGB from Settings → Apps if it remains".into());
    }
    notes.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_lists_friendly_names() {
        std::env::set_var("CHROMAFLOW_CONFLICTS", "Fan Control,Corsair iCUE");
        assert_eq!(detect(), vec!["Fan Control".to_string(), "Corsair iCUE".to_string()]);
        std::env::set_var("CHROMAFLOW_CONFLICTS", "none");
        assert!(detect().is_empty());
        std::env::remove_var("CHROMAFLOW_CONFLICTS");
    }

    #[test]
    fn catalog_has_fan_control() {
        assert!(CATALOG.iter().any(|(_, d)| *d == "Fan Control"));
    }

    #[test]
    fn catalog_kills_ggez_launcher() {
        let ss = CATALOG
            .iter()
            .find(|(_, d)| *d == "SteelSeries GG")
            .expect("SteelSeries GG row");
        assert!(ss.0.iter().any(|img| *img == "SteelSeriesGGEZ"));
    }

    #[test]
    fn gigabyte_suppress_targets_easytune_and_fancontrol() {
        let src = include_str!("conflicts.rs");
        assert!(src.contains("suppress_gigabyte_fan_autostart"));
        assert!(src.contains("EasyTuneEngineService"));
        assert!(src.contains("FanControl.Service"));
        assert!(src.contains("schtasks"));
    }
}
