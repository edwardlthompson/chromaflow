//! Color window icon for the Linux panel and Windows taskbar. No PWM.

use tauri::image::Image;
use tauri::Manager;

const ICON_PNG: &[u8] = include_bytes!("../icons/48x48.png");

pub fn apply(app: &tauri::App) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    if let Ok(icon) = Image::from_bytes(ICON_PNG) {
        let _ = win.set_icon(icon);
        return;
    }
    if let Some(icon) = app.default_window_icon() {
        let _ = win.set_icon(icon.clone());
    }
}
