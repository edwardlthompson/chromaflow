//! Enable the user chromaflowd watchdog so duty survives GUI close.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn user_unit() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
        });
    base.join("systemd/user/chromaflowd.service")
}

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

pub fn is_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "chromaflowd.service"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn enable() -> Result<String, String> {
    seed_unit()?;
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    systemctl(&["--user", "enable", "--now", "chromaflowd.service"])?;
    Ok("chromaflowd enabled".into())
}

pub fn disable() -> Result<(), String> {
    let _ = systemctl(&["--user", "disable", "--now", "chromaflowd.service"]);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn user_unit_is_under_config() {
        let p = super::user_unit();
        assert!(p.ends_with("systemd/user/chromaflowd.service"));
    }
}
