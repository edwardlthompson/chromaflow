//! One chromaflow-gui per user. Second launch focuses the first window.

use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use tauri::Manager;

#[cfg(unix)]
fn sock_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        format!(
            "/tmp/chromaflow-{}",
            std::env::var("USER").unwrap_or_else(|_| "user".into())
        )
    });
    PathBuf::from(dir).join("chromaflow-gui.sock")
}

/// Bind the session lock, or signal the running GUI and return None.
#[cfg(unix)]
pub fn bind() -> Option<UnixListener> {
    let path = sock_path();
    if UnixStream::connect(&path).is_ok() {
        return None;
    }
    let _ = fs::remove_file(&path);
    UnixListener::bind(&path).ok()
}

#[cfg(unix)]
pub fn watch(app: AppHandle, listener: UnixListener) {
    thread::spawn(move || {
        for stream in listener.incoming() {
            drop(stream);
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.unminimize();
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    });
}

/// Windows: lock file under LOCALAPPDATA. Second instance exits (no focus IPC yet).
#[cfg(windows)]
pub struct InstanceLock {
    _file: fs::File,
    path: PathBuf,
}

#[cfg(windows)]
fn lock_path() -> PathBuf {
    let dir = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("chromaflow");
    let _ = fs::create_dir_all(&dir);
    dir.join("chromaflow-gui.lock")
}

#[cfg(windows)]
pub fn bind() -> Option<InstanceLock> {
    let path = lock_path();
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => Some(InstanceLock { _file: file, path }),
        Err(_) => {
            // Stale lock from a crash: try remove and retry once.
            let _ = fs::remove_file(&path);
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .ok()
                .map(|file| InstanceLock { _file: file, path })
        }
    }
}

#[cfg(windows)]
pub fn watch(_app: AppHandle, _lock: InstanceLock) {
    // Lock file held for process lifetime; Drop removes it.
}

#[cfg(windows)]
impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
