//! Enable the chromaflowd watchdog so duty survives GUI close.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(windows))]
fn user_unit() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
        });
    base.join("systemd/user/chromaflowd.service")
}

#[cfg(not(windows))]
fn seed_unit() -> Result<(), String> {
    if PathBuf::from("/usr/lib/systemd/user/chromaflowd.service").is_file() {
        return Ok(());
    }
    let dest = user_unit();
    if dest.is_file() {
        return Ok(());
    }
    let src = std::env::var("CHROMAFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging"))
        .join("chromaflowd.service");
    if !src.is_file() {
        return Err("chromaflowd.service missing".into());
    }
    fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
fn systemctl(args: &[&str]) -> Result<(), String> {
    let out = Command::new("systemctl")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(windows)]
fn marker_path() -> PathBuf {
    crate::paths::runtime_dir().join("chromaflowd.active")
}

pub fn is_active() -> bool {
    #[cfg(windows)]
    {
        if marker_path().is_file() {
            return true;
        }
        let mut cmd = Command::new("sc");
        cmd.args(["query", "chromaflowd"]);
        crate::win_cmd::hidden_capture(cmd)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("RUNNING"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", "chromaflowd.service"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(windows)]
fn sc_query_text() -> String {
    let mut query = Command::new("sc");
    query.args(["query", "chromaflowd"]);
    crate::win_cmd::hidden_capture(query)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

#[cfg(windows)]
fn service_running(text: &str) -> bool {
    text.contains("RUNNING")
}

#[cfg(windows)]
fn service_exists(text: &str) -> bool {
    text.contains("SERVICE_NAME")
}

/// `sc start` can block forever on START_PENDING — never wait more than a few seconds.
#[cfg(windows)]
fn start_service_bounded(secs: u64) -> String {
    let text = sc_query_text();
    if !service_exists(&text) {
        return "chromaflowd service not registered (session marker only)".into();
    }
    if service_running(&text) {
        return "chromaflowd service already RUNNING".into();
    }
    let mut start = Command::new("sc");
    start.args(["start", "chromaflowd"]);
    let mut child = match crate::win_cmd::hidden(start).spawn() {
        Ok(c) => c,
        Err(e) => return format!("sc start spawn failed: {e}"),
    };
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let after = sc_query_text();
                return if service_running(&after) {
                    format!("sc start ok ({status})")
                } else {
                    format!("sc start finished ({status}); state not RUNNING yet")
                };
            }
            Ok(None) if Instant::now() < deadline => {
                if service_running(&sc_query_text()) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return "chromaflowd reached RUNNING (sc start still pending — abandoned)".into();
                }
                thread::sleep(Duration::from_millis(200));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let after = sc_query_text();
                return if service_running(&after) {
                    "chromaflowd RUNNING after timed-out sc start".into()
                } else {
                    format!(
                        "sc start timed out after {secs}s (often START_PENDING) — marker kept; watchdog may still run as a process"
                    )
                };
            }
        }
    }
}

pub fn enable() -> Result<String, String> {
    #[cfg(windows)]
    {
        let path = marker_path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        fs::write(&path, b"1\n").map_err(|e| e.to_string())?;
        let detail = start_service_bounded(8);
        Ok(format!("chromaflowd enabled (session marker); {detail}"))
    }
    #[cfg(not(windows))]
    {
        seed_unit()?;
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        systemctl(&["--user", "enable", "--now", "chromaflowd.service"])?;
        Ok("chromaflowd enabled".into())
    }
}

pub fn disable() -> Result<(), String> {
    #[cfg(windows)]
    {
        let _ = fs::remove_file(marker_path());
        let mut stop = Command::new("sc");
        stop.args(["stop", "chromaflowd"]);
        let _ = crate::win_cmd::hidden(stop).status();
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = systemctl(&["--user", "disable", "--now", "chromaflowd.service"]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn enable_disable_round_trip_smoke() {
        // Do not touch real systemd/sc in unit tests beyond is_active false default.
        let _ = super::is_active();
    }

    #[cfg(not(windows))]
    #[test]
    fn user_unit_is_under_config() {
        let p = super::user_unit();
        assert!(p.ends_with("systemd/user/chromaflowd.service"));
    }
}
