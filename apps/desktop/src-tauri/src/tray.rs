//! Color tray launcher. Open/Quit only. No PWM.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

const ICON_PNG: &[u8] = include_bytes!("../icons/32x32.png");
static QUIT: AtomicBool = AtomicBool::new(false);

pub fn quitting() -> bool {
    QUIT.load(Ordering::SeqCst)
}

fn show(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

pub fn attach(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open ChromaFlow", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let icon = Image::from_bytes(ICON_PNG)?;
    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("ChromaFlow")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show(app),
            "quit" => {
                QUIT.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
