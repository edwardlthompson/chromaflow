//! Spawn a sibling OpenRGB SDK server on 127.0.0.1. Never Flatpak/Wine. No PWM.

use crate::openrgb_engine::{dest_path, share_dir};
use crate::openrgb_proto::{connect, lock_sdk};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

static SPAWNING: AtomicBool = AtomicBool::new(false);
/// First ensure_local_server in this process always restarts so OpenRGB.json
/// (run_zone_checks=false) is loaded — avoids a stale in-memory Zone Initialization dialog.
static SETTINGS_RELOADED: AtomicBool = AtomicBool::new(false);
static CHILD: Mutex<Option<Child>> = Mutex::new(None);
static HOLD_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);

pub fn sdk_opt_in() -> bool {
    match std::env::var("CHROMAFLOW_OPENRGB_SDK") {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        // Windows: inventory/paint against a local SDK without requiring an env flag.
        Err(_) => cfg!(windows),
    }
}

pub fn spawn_allowed() -> bool {
    if !sdk_opt_in() {
        return false;
    }
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

#[cfg_attr(windows, allow(dead_code))]
fn which_openrgb() -> Option<PathBuf> {
    let path = std::env::var("PATH").ok()?;
    #[cfg(windows)]
    let sep = ';';
    #[cfg(not(windows))]
    let sep = ':';
    #[cfg(windows)]
    let names = ["OpenRGB.exe", "openrgb.exe"];
    #[cfg(not(windows))]
    let names = ["openrgb"];
    for dir in path.split(sep) {
        for name in names {
            let p = PathBuf::from(dir).join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn windows_install_candidates() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut out = Vec::new();
        if let Ok(pf) = std::env::var("ProgramFiles") {
            out.push(
                PathBuf::from(&pf)
                    .join("ChromaFlow")
                    .join("engine")
                    .join("OpenRGB")
                    .join("OpenRGB.exe"),
            );
            // Legacy flat layout from early Windows builds.
            out.push(PathBuf::from(&pf).join("ChromaFlow").join("OpenRGB.exe"));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            out.push(
                PathBuf::from(&local)
                    .join("chromaflow")
                    .join("engine")
                    .join("OpenRGB")
                    .join("OpenRGB.exe"),
            );
            out.push(
                PathBuf::from(&local)
                    .join("chromaflow")
                    .join("OpenRGB.exe"),
            );
        }
        out
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

pub fn is_private_engine_path(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase().replace('/', "\\");
    s.contains("chromaflow\\engine\\") || s.contains("chromaflow\\openrgb")
}

fn private_bin_candidates() -> Vec<PathBuf> {
    let mut cands = Vec::new();
    cands.push(crate::openrgb_engine::packaged_engine_path());
    cands.push(dest_path());
    cands.extend(windows_install_candidates());
    cands
}

pub fn resolve_bin() -> Option<PathBuf> {
    // Prefer the private ChromaFlow engine over CHROMAFLOW_OPENRGB (seed source)
    // or a user-installed OpenRGB, so we never launch a standalone GUI copy.
    let mut cands = private_bin_candidates();
    if let Ok(p) = std::env::var("CHROMAFLOW_OPENRGB") {
        cands.push(PathBuf::from(p));
    }
    #[cfg(not(windows))]
    if !crate::openrgb_sandbox::sandboxed() {
        if let Some(p) = which_openrgb() {
            cands.push(p);
        }
    }
    // Windows: never fall through to PATH OpenRGB when any private candidate exists on disk.
    #[cfg(windows)]
    {
        if let Some(p) = cands.iter().find(|p| p.is_file() && !forbidden(p)).cloned() {
            return Some(p);
        }
        // Last resort only when no private engine is installed yet.
        if !crate::openrgb_sandbox::sandboxed() {
            if let Some(p) = which_openrgb() {
                if p.is_file() && !forbidden(&p) {
                    return Some(p);
                }
            }
        }
        return None;
    }
    #[cfg(not(windows))]
    {
        cands.into_iter().find(|p| p.is_file() && !forbidden(p))
    }
}

/// SDK server argv — headless only. Do not pass the tray/minimized GUI flag (implies `--gui`).
pub fn server_args() -> &'static [&'static str] {
    &[
        "--server",
        "--server-host",
        "127.0.0.1",
        "--server-port",
        "6742",
        "--noautoconnect",
    ]
}

/// Kill OpenRGB.exe processes that are not under ChromaFlow\\engine (standalone tray hijack).
pub fn kill_non_private_openrgb() -> String {
    #[cfg(windows)]
    {
        let script = "Get-CimInstance Win32_Process -Filter \"Name='OpenRGB.exe'\" | \
            ForEach-Object { \
              $p = $_.ExecutablePath; \
              if (-not $p) { return }; \
              $l = $p.ToLower(); \
              if ($l -like '*chromaflow*engine*' -or $l -like '*chromaflow\\openrgb*') { return }; \
              Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue; \
              Write-Output $p \
            }";
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
        let out = crate::win_cmd::hidden_capture(cmd).output();
        let text = out
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if text.is_empty() {
            "no standalone OpenRGB process".into()
        } else {
            format!("stopped standalone OpenRGB: {}", text.replace('\n', "; "))
        }
    }
    #[cfg(not(windows))]
    {
        "n/a".into()
    }
}

/// Clear standalone OpenRGB Run / Startup entries so logon does not relaunch a tray app.
pub fn clear_openrgb_autostart() -> String {
    #[cfg(windows)]
    {
        let mut notes = Vec::new();
        for (hive, label) in [
            (r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "HKCU"),
            (r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run", "HKLM"),
        ] {
            for name in ["OpenRGB", "openrgb", "OpenRGB.OpenRGB"] {
                let mut cmd = Command::new("reg");
                cmd.args(["delete", hive, "/v", name, "/f"]);
                if crate::win_cmd::hidden(cmd)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    notes.push(format!("cleared {label} Run\\{name}"));
                }
            }
        }
        let script = "$dirs = @(\
            [Environment]::GetFolderPath('Startup'),\
            [Environment]::GetFolderPath('CommonStartup')\
          ); \
          foreach ($d in $dirs) { \
            if (-not $d) { continue }; \
            Get-ChildItem -LiteralPath $d -Filter '*OpenRGB*' -ErrorAction SilentlyContinue | \
              ForEach-Object { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue; $_.Name } \
          }";
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
        if let Ok(o) = crate::win_cmd::hidden_capture(cmd).output() {
            let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !t.is_empty() {
                notes.push(format!("removed Startup: {}", t.replace('\n', "; ")));
            }
        }
        if notes.is_empty() {
            "no OpenRGB Run/Startup entries".into()
        } else {
            notes.join("; ")
        }
    }
    #[cfg(not(windows))]
    {
        "n/a".into()
    }
}

fn listening_is_private() -> bool {
    #[cfg(windows)]
    {
        let script = "$p = Get-CimInstance Win32_Process -Filter \"Name='OpenRGB.exe'\" | \
            Select-Object -First 1 -ExpandProperty ExecutablePath; \
            if (-not $p) { '' } else { $p }";
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ]);
        let path = crate::win_cmd::hidden_capture(cmd)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if path.is_empty() {
            // Port open but no OpenRGB.exe named — treat as unknown; force restart.
            return false;
        }
        is_private_engine_path(Path::new(&path))
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Start a local OpenRGB SDK server when the binary is known and :6742 is down.
pub fn ensure_local_server() -> Result<String, String> {
    // Rewrite settings first; restart when JSON changed or first call this process
    // so run_zone_checks=false is actually loaded (stale process shows Zone Initialization).
    let settings_dirty = crate::openrgb_engine::configure_engine_ui().unwrap_or(false);
    let first_load = !SETTINGS_RELOADED.swap(true, Ordering::SeqCst);
    let must_reload = settings_dirty || first_load;
    if crate::openrgb_proto::connect().is_ok() {
        if listening_is_private() && !must_reload {
            return Ok("private lighting engine already on 127.0.0.1:6742".into());
        }
        let _ = kill_non_private_openrgb();
        let _ = clear_openrgb_autostart();
        let _ = restart_sdk_force_stop();
        thread::sleep(Duration::from_millis(500));
    }
    let _ = kill_non_private_openrgb();
    let _ = clear_openrgb_autostart();
    let bin = resolve_bin().ok_or_else(|| {
        "Lighting engine not found. Use Support → Install lighting engine (private OpenRGB under ChromaFlow\\engine)."
            .to_string()
    })?;
    if !is_private_engine_path(&bin) && private_bin_candidates().iter().any(|p| p.is_file()) {
        return Err(format!(
            "refusing to start non-private OpenRGB at {}",
            bin.display()
        ));
    }
    let _ = crate::openrgb_engine::configure_engine_ui();
    let mut cmd = Command::new(&bin);
    cmd.args(server_args());
    let child = crate::win_cmd::hidden(cmd)
        .spawn()
        .map_err(|e| format!("spawn lighting engine: {e}"))?;
    let child_pid = child.id();
    if let Ok(mut g) = CHILD.lock() {
        *g = Some(child);
    }
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if crate::openrgb_proto::connect().is_ok() {
            // Port opens mid-detection; wait for at least one controller (Keychron/MB/GPU).
            match crate::openrgb_proto::wait_for_controllers(Duration::from_secs(12)) {
                Ok(n) => {
                    let _ = fs::write(pid_path(), format!("{child_pid}"));
                    return Ok(format!(
                        "started {} — SDK 127.0.0.1:6742 ({n} controllers)",
                        bin.display()
                    ));
                }
                Err(_) => {
                    // Keep waiting until overall deadline; detection can exceed 12s cold.
                    if Instant::now() + Duration::from_secs(2) >= deadline {
                        break;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err("OpenRGB started but SDK did not list controllers in time".into())
}

/// Stop OpenRGB via pid file / child handle without immediately respawning (Windows boot uses ensure_local_server).
fn restart_sdk_force_stop() -> bool {
    {
        let mut g = CHILD.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(mut ch) = g.take() {
            let _ = ch.kill();
            let _ = ch.wait();
        }
    }
    if let Some(pid) = sibling_pid() {
        if proc_running(pid) {
            #[cfg(windows)]
            {
                let mut kill = Command::new("taskkill");
                kill.args(["/PID", &pid.to_string(), "/F"]);
                let _ = crate::win_cmd::hidden(kill).status();
            }
            #[cfg(not(windows))]
            {
                let _ = Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();
            }
            for _ in 0..25 {
                if !proc_running(pid) {
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    }
    let _ = kill_non_private_openrgb();
    let _ = fs::remove_file(pid_path());
    SPAWNING.store(false, Ordering::SeqCst);
    *HOLD_UNTIL.lock().unwrap_or_else(|p| p.into_inner()) = None;
    crate::openrgb::drop_probe_cache();
    true
}

fn pid_path() -> PathBuf {
    share_dir().join("openrgb.pid")
}

fn sibling_pid() -> Option<u32> {
    let raw = fs::read_to_string(pid_path()).ok()?;
    let pid: u32 = raw.trim().parse().ok()?;
    (pid >= 2).then_some(pid)
}

#[cfg_attr(windows, allow(dead_code))]
fn proc_state(stat: &str) -> Option<char> {
    stat.rsplit_once(')')?.1.trim().chars().next()
}

fn proc_running(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("tasklist");
        cmd.args(["/FI", &format!("PID eq {pid}"), "/NH"]);
        crate::win_cmd::hidden_capture(cmd)
            .output()
            .map(|o| {
                let t = String::from_utf8_lossy(&o.stdout);
                t.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        let text = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
        matches!(proc_state(&text), Some(s) if s != 'Z' && s != 'X')
    }
}

fn reap_child() {
    let mut g = CHILD.lock().unwrap_or_else(|p| p.into_inner());
    let Some(ch) = g.as_mut() else {
        return;
    };
    match ch.try_wait() {
        Ok(Some(_)) | Err(_) => {
            *g = None;
            let _ = fs::remove_file(pid_path());
        }
        Ok(None) => {}
    }
}

fn pid_alive() -> bool {
    reap_child();
    if CHILD.lock().unwrap_or_else(|p| p.into_inner()).is_some() {
        return true;
    }
    sibling_pid().is_some_and(proc_running)
}

fn spawn_held() -> bool {
    let g = HOLD_UNTIL.lock().unwrap_or_else(|p| p.into_inner());
    matches!(*g, Some(t) if Instant::now() < t)
}

fn hold_spawn() {
    *HOLD_UNTIL.lock().unwrap_or_else(|p| p.into_inner()) =
        Some(Instant::now() + Duration::from_secs(30));
}

fn store_child(child: Child) {
    let mut g = CHILD.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(mut old) = g.take() {
        let _ = old.kill();
        let _ = old.wait();
    }
    *g = Some(child);
}

fn apply_x11(cmd: &mut Command) {
    let dpy = std::env::var("CHROMAFLOW_DISPLAY")
        .or_else(|_| std::env::var("DISPLAY"))
        .unwrap_or_else(|_| ":0".into());
    cmd.env("DISPLAY", &dpy);
    if std::env::var_os("XAUTHORITY").is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let xa = PathBuf::from(home).join(".Xauthority");
            if xa.is_file() {
                cmd.env("XAUTHORITY", xa);
            }
        }
    }
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
    let _ = crate::openrgb_engine::configure_engine_ui();
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
    apply_x11(&mut cmd);
    if bin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("AppImage"))
    {
        cmd.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
    cmd.args(server_args())
    .stdin(Stdio::null())
    .stdout(Stdio::from(log))
    .stderr(Stdio::from(err_log));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let child = cmd
        .spawn()
        .map_err(|e| fuse_hint("", &e.to_string()))?;
    let _ = fs::write(pid_path(), format!("{}\n", child.id()));
    store_child(child);
    if wait_port() {
        *HOLD_UNTIL.lock().unwrap_or_else(|p| p.into_inner()) = None;
        return Ok(());
    }
    reap_child();
    hold_spawn();
    let log_txt = fs::read_to_string(log_path).unwrap_or_default();
    Err(fuse_hint(
        &log_txt,
        "OpenRGB engine started but 127.0.0.1:6742 stayed down",
    ))
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
    reap_child();
    if pid_alive() {
        let _ = wait_port();
        return;
    }
    if spawn_held() {
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

/// SIGTERM the OpenRGB pid file only. Then spawn again.
pub fn restart_sdk_once() -> bool {
    if !spawn_allowed() {
        return false;
    }
    {
        let mut g = CHILD.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(mut ch) = g.take() {
            let _ = ch.kill();
            let _ = ch.wait();
        }
    }
    if let Some(pid) = sibling_pid() {
        if proc_running(pid) {
            #[cfg(windows)]
            {
                let mut kill = Command::new("taskkill");
                kill.args(["/PID", &pid.to_string(), "/F"]);
                let _ = crate::win_cmd::hidden(kill).status();
            }
            #[cfg(not(windows))]
            {
                let _ = Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();
            }
            for _ in 0..25 {
                if !proc_running(pid) {
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
            if proc_running(pid) {
                return false;
            }
        }
    }
    let _ = fs::remove_file(pid_path());
    SPAWNING.store(false, Ordering::SeqCst);
    *HOLD_UNTIL.lock().unwrap_or_else(|p| p.into_inner()) = None;
    crate::openrgb::drop_probe_cache();
    #[cfg(windows)]
    {
        let _ = ensure_local_server();
    }
    #[cfg(not(windows))]
    {
        ensure_sdk();
    }
    true
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
        // Isolate from a real private engine under the user's LocalAppData.
        std::env::set_var("LOCALAPPDATA", dir.join("local"));
        std::env::set_var("ProgramFiles", dir.join("pf"));
        std::env::set_var("CHROMAFLOW_OPENRGB", &env_bin);
        std::env::set_var("CHROMAFLOW_DATA_HOME", dir.join("share"));
        assert_eq!(resolve_bin().as_deref(), Some(env_bin.as_path()));
        std::env::set_var(
            "CHROMAFLOW_OPENRGB",
            "/var/lib/flatpak/app/org.openrgb.OpenRGB/openrgb",
        );
        assert_ne!(
            resolve_bin()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            "/var/lib/flatpak/app/org.openrgb.OpenRGB/openrgb"
        );
        std::env::set_var("CHROMAFLOW_OPENRGB_CMDLINE", "bwrap -- openrgb --server");
        std::env::remove_var("CHROMAFLOW_OPENRGB");
        let _ = resolve_bin();
        std::env::remove_var("CHROMAFLOW_OPENRGB_CMDLINE");
        let marker = dir.join("spawned");
        let _ = fs::remove_file(&marker);
        let stub = dir.join("stub-openrgb");
        fs::write(
            &stub,
            format!("#!/bin/sh\necho ran > {}\n", marker.display()),
        )
        .unwrap();
        crate::fs_mode::set_mode(&stub, 0o755).unwrap();
        std::env::set_var("CHROMAFLOW_OPENRGB", &stub);
        std::env::set_var("CHROMAFLOW_NO_SPAWN", "1");
        std::env::set_var("CHROMAFLOW_ALLOW_SPAWN", "1");
        ensure_sdk();
        assert!(!marker.exists());
        assert!(!restart_sdk_once());
        std::env::remove_var("CHROMAFLOW_NO_SPAWN");
        std::env::remove_var("CHROMAFLOW_ALLOW_SPAWN");
        std::env::remove_var("CHROMAFLOW_OPENRGB");
        assert!(!forbidden(Path::new(
            "/usr/libexec/chromaflow/OpenRGB.AppImage"
        )));
        assert!(forbidden(Path::new("/home/u/.wine/openrgb.exe")));
        assert_eq!(proc_state("3116 (OpenRGB) Z 2746"), Some('Z'));
        assert_eq!(proc_state("3116 (OpenRGB) S 2746"), Some('S'));
        assert!(!matches!(proc_state("3116 (OpenRGB) Z 2746"), Some(s) if s != 'Z' && s != 'X'));
        let _ = engine_missing();
    }

    #[test]
    fn headless_server_args_no_tray() {
        let args = server_args();
        assert!(args.contains(&"--server"));
        assert!(args.contains(&"--noautoconnect"));
        assert!(args.contains(&"127.0.0.1"));
        assert!(!args.iter().any(|a| a.contains("minimized")));
        assert!(!args.iter().any(|a| *a == "--gui"));
    }
}
