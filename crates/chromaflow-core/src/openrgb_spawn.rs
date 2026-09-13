//! Spawn a sibling OpenRGB SDK server on 127.0.0.1. Never Flatpak/Wine. No PWM.

use crate::openrgb_engine::{dest_path, share_dir};
use crate::openrgb_proto::{connect, lock_sdk};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static SPAWNING: AtomicBool = AtomicBool::new(false);

pub fn spawn_allowed() -> bool {
    if std::env::var("CHROMAFLOW_NO_SPAWN").ok().as_deref() == Some("1") {
        return false;
    }
    if cfg!(test) && std::env::var("CHROMAFLOW_ALLOW_SPAWN").ok().as_deref() != Some("1") {
        return false;
    }
    true
}

fn forbidden(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase();
    s.contains("flatpak") || s.contains("bwrap") || s.contains("wine")
}

fn which_openrgb() -> Option<PathBuf> {
    let path = std::env::var("PATH").ok()?;
    path.split(':')
        .map(|d| PathBuf::from(d).join("openrgb"))
        .find(|p| p.is_file())
}

pub fn resolve_bin() -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Ok(p) = std::env::var("CHROMAFLOW_OPENRGB") {
        cands.push(PathBuf::from(p));
    }
    cands.push(PathBuf::from("/usr/libexec/chromaflow/OpenRGB.AppImage"));
    cands.push(dest_path());
    if !crate::openrgb_sandbox::sandboxed() {
        if let Some(p) = which_openrgb() {
            cands.push(p);
        }
    }
    cands.into_iter().find(|p| p.is_file() && !forbidden(p))
}

fn pid_path() -> PathBuf {
    share_dir().join("openrgb.pid")
}

fn pid_alive() -> bool {
    let Ok(raw) = fs::read_to_string(pid_path()) else {
        return false;
    };
    let Ok(pid) = raw.trim().parse::<u32>() else {
        return false;
    };
    Path::new(&format!("/proc/{pid}")).exists()
}

fn wait_port() -> bool {
    for _ in 0..10 {
        if connect().is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(200));
    }
    false
}

fn fuse_hint(log: &str, err: &str) -> String {
    let blob = format!("{log} {err}").to_ascii_lowercase();
    if blob.contains("fuse") {
        "OpenRGB AppImage needs FUSE. Run: sudo apt install libfuse2".into()
    } else {
        err.to_string()
    }
}

fn spawn_bin(bin: &Path) -> Result<(), String> {
    let dir = share_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let log_path = dir.join("openrgb.log");
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    let err_log = log.try_clone().map_err(|e| e.to_string())?;
    let mut cmd = Command::new(bin);
    if bin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("AppImage"))
    {
        cmd.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
    let child = cmd
        .args([
            "--server",
            "--server-host",
            "127.0.0.1",
            "--noautoconnect",
            "--startminimized",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(err_log))
        .spawn()
        .map_err(|e| fuse_hint("", &e.to_string()))?;
    let _ = fs::write(pid_path(), format!("{}\n", child.id()));
    std::mem::forget(child);
    if wait_port() {
        return Ok(());
    }
    let log_txt = fs::read_to_string(log_path).unwrap_or_default();
    Err(fuse_hint(&log_txt, "OpenRGB engine started but 127.0.0.1:6742 stayed down"))
}

/// Reuse 6742 if up; otherwise spawn at most one sibling. Missing binary is a no-op.
pub fn ensure_sdk() {
    if connect().is_ok() {
        return;
    }
    if !spawn_allowed() {
        return;
    }
    let _sdk = lock_sdk();
    if connect().is_ok() {
        return;
    }
    if pid_alive() {
        let _ = wait_port();
        return;
    }
    let Some(bin) = resolve_bin() else {
        return;
    };
    if SPAWNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        let _ = wait_port();
        return;
    }
    let _ = spawn_bin(&bin);
    SPAWNING.store(false, Ordering::SeqCst);
}

pub fn engine_missing() -> bool {
    connect().is_err() && resolve_bin().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_forbid_and_no_spawn() {
        let dir = std::env::temp_dir().join("cf-spawn-order");
        let _ = fs::create_dir_all(&dir);
        let env_bin = dir.join("env-openrgb");
        fs::write(&env_bin, b"#!/bin/sh\nexit 0\n").unwrap();
        std::env::set_var("CHROMAFLOW_OPENRGB", &env_bin);
        std::env::set_var("CHROMAFLOW_DATA_HOME", dir.join("share"));
        assert_eq!(resolve_bin().as_deref(), Some(env_bin.as_path()));
        std::env::set_var(
            "CHROMAFLOW_OPENRGB",
            "/var/lib/flatpak/app/org.openrgb.OpenRGB/openrgb",
        );
        assert_ne!(
            resolve_bin().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
            "/var/lib/flatpak/app/org.openrgb.OpenRGB/openrgb"
        );
        std::env::set_var("CHROMAFLOW_OPENRGB_CMDLINE", "bwrap -- openrgb --server");
        std::env::remove_var("CHROMAFLOW_OPENRGB");
        let _ = resolve_bin();
        std::env::remove_var("CHROMAFLOW_OPENRGB_CMDLINE");
        let marker = dir.join("spawned");
        let _ = fs::remove_file(&marker);
        let stub = dir.join("stub-openrgb");
        fs::write(&stub, format!("#!/bin/sh\necho ran > {}\n", marker.display())).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&stub).unwrap().permissions();
        p.set_mode(0o755);
        fs::set_permissions(&stub, p).unwrap();
        std::env::set_var("CHROMAFLOW_OPENRGB", &stub);
        std::env::set_var("CHROMAFLOW_NO_SPAWN", "1");
        std::env::set_var("CHROMAFLOW_ALLOW_SPAWN", "1");
        ensure_sdk();
        assert!(!marker.exists());
        std::env::remove_var("CHROMAFLOW_NO_SPAWN");
        std::env::remove_var("CHROMAFLOW_ALLOW_SPAWN");
        std::env::remove_var("CHROMAFLOW_OPENRGB");
        assert!(!forbidden(Path::new("/usr/libexec/chromaflow/OpenRGB.AppImage")));
        assert!(forbidden(Path::new("/home/u/.wine/openrgb.exe")));
        let _ = engine_missing();
    }
}
