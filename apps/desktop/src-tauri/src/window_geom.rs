//! Persist main window size across launches. No PWM.

use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{Manager, PhysicalSize, Size, WindowEvent};

const MIN_W: u32 = 720;
const MIN_H: u32 = 520;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geom {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub maximized: bool,
}

impl Default for Geom {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            maximized: false,
        }
    }
}

fn path() -> std::path::PathBuf {
    chromaflow_core::profiles::config_dir().join("window.json")
}

pub fn clamp(mut g: Geom) -> Geom {
    g.width = g.width.max(MIN_W);
    g.height = g.height.max(MIN_H);
    g
}

pub fn load() -> Option<Geom> {
    let raw = fs::read_to_string(path()).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save(g: &Geom) -> Result<(), String> {
    let dir = chromaflow_core::profiles::config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(&clamp(g.clone())).map_err(|e| e.to_string())?;
    fs::write(path(), body).map_err(|e| e.to_string())
}

pub fn restore(app: &tauri::App) {
    let Some(g) = load() else {
        return;
    };
    let g = clamp(g);
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let _ = win.set_size(Size::Physical(PhysicalSize {
        width: g.width,
        height: g.height,
    }));
    if g.maximized {
        let _ = win.maximize();
    }
}

pub fn on_event(window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::Resized(_) | WindowEvent::CloseRequested { .. } => {
            if let Ok(size) = window.inner_size() {
                let maximized = window.is_maximized().unwrap_or(false);
                let _ = save(&Geom {
                    width: size.width,
                    height: size.height,
                    maximized,
                });
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn clamp_floor() {
        let g = super::clamp(super::Geom {
            width: 10,
            height: 10,
            maximized: false,
        });
        assert!(g.width >= 720);
        assert!(g.height >= 520);
    }
}
