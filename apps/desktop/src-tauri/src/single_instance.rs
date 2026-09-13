//! One chromaflow-gui per user. Second launch focuses the first window.

use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread;
use tauri::{AppHandle, Manager};

fn sock_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        format!("/tmp/chromaflow-{}", std::env::var("USER").unwrap_or_else(|_| "user".into()))
    });
    PathBuf::from(dir).join("chromaflow-gui.sock")
}

/// Bind the session socket, or ping the running GUI and return None.
pub fn bind() -> Option<UnixListener> {
    let path = sock_path();
    if UnixStream::connect(&path).is_ok() {
        return None;
    }
    let _ = fs::remove_file(&path);
    UnixListener::bind(&path).ok()
}

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
