//! Unprivileged Tauri 2 shell. Refuses root. Support, lighting, PWM watchdog.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chromaflow_core::gauges;
use chromaflow_core::hwmon;
use chromaflow_core::lighting;
use chromaflow_core::lighting_apply as color_apply;
use chromaflow_core::openrgb_engine;
use chromaflow_core::profiles;
use chromaflow_core::pwm_apply;
use chromaflow_core::pwm_calibrate as pwm_cal;
use chromaflow_core::pwm_curves;
use chromaflow_core::pwm_daemon;
use chromaflow_core::refuse_if_root;
use chromaflow_core::support;
use chromaflow_core::update_fetch;
use chromaflow_core::{collect_cooling, collect_inventory};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

mod icon;
mod session;
mod single_instance;
mod tray;
mod window_geom;

fn repo_root() -> PathBuf {
    std::env::var("CHROMAFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

#[tauri::command]
fn support_dry_run(advanced: bool) -> Result<Value, String> {
    refuse_if_root()?;
    let plan = support::dry_run(&repo_root(), advanced)?;
    Ok(plan.plan)
}

#[tauri::command]
fn support_apply(advanced: bool, extra: Option<String>) -> Result<Value, String> {
    refuse_if_root()?;
    support::apply(advanced, extra.as_deref())
}

#[tauri::command]
fn competitors_plan() -> Result<Value, String> {
    refuse_if_root()?;
    support::competitors_plan(&repo_root())
}

#[tauri::command]
fn competitors_remove() -> Result<Value, String> {
    refuse_if_root()?;
    support::competitors_remove()
}

#[tauri::command]
fn inventory(light: Option<bool>) -> Result<Value, String> {
    refuse_if_root()?;
    if light.unwrap_or(false) {
        serde_json::to_value(collect_inventory()).map_err(|e| e.to_string())
    } else {
        serde_json::to_value(collect_cooling()).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn hardware_gauges() -> Result<Value, String> {
    refuse_if_root()?;
    let chips = hwmon::scan_temps(&hwmon::hwmon_root());
    serde_json::to_value(gauges::snapshot(&chips)).map_err(|e| e.to_string())
}

#[tauri::command]
fn profiles_load() -> Result<Value, String> {
    refuse_if_root()?;
    let file = profiles::load(&profiles::config_dir())?;
    serde_json::to_value(file).map_err(|e| e.to_string())
}

#[tauri::command]
fn lighting_preview() -> Result<Value, String> {
    refuse_if_root()?;
    serde_json::to_value(chromaflow_core::openrgb_preview::snapshot()).map_err(|e| e.to_string())
}

#[tauri::command]
fn lighting_sync(frames: Vec<chromaflow_core::openrgb_preview::LedFrame>) -> Result<Value, String> {
    refuse_if_root()?;
    let pushing = !frames.is_empty();
    let rows = chromaflow_core::openrgb_preview::sync(frames);
    if pushing {
        return Ok(Value::Array(vec![]));
    }
    serde_json::to_value(rows).map_err(|e| e.to_string())
}

#[tauri::command]
fn lighting_broadcast(color: String, mode: String) -> Result<Value, String> {
    refuse_if_root()?;
    let detail = chromaflow_core::lighting_broadcast::broadcast(&color, &mode)?;
    Ok(serde_json::json!({ "ok": true, "detail": detail }))
}

#[tauri::command]
fn lighting_cycle(on: bool, speed: u8) -> Result<Value, String> {
    refuse_if_root()?;
    chromaflow_core::lighting_cycle::set_running(on, speed);
    Ok(serde_json::json!({ "ok": true, "on": on }))
}

#[tauri::command]
fn lighting_apply(
    backend: String,
    device: String,
    color: String,
    mode: Option<String>,
    led: Option<u16>,
) -> Result<Value, String> {
    refuse_if_root()?;
    let detail = if let Some(led) = led {
        color_apply::apply_led(&backend, &device, led, &color)?
    } else if let Some(mode) = mode.filter(|m| !m.trim().is_empty()) {
        color_apply::apply_mode(&backend, &device, &mode, &color)?
    } else {
        color_apply::apply(&backend, &device, &color)?
    };
    Ok(serde_json::json!({ "ok": true, "detail": detail }))
}

#[tauri::command]
fn session_load() -> Result<Value, String> {
    refuse_if_root()?;
    serde_json::to_value(session::load()).map_err(|e| e.to_string())
}

#[tauri::command]
fn session_save(session: session::Session) -> Result<Value, String> {
    refuse_if_root()?;
    serde_json::to_value(session::save(session)?).map_err(|e| e.to_string())
}

#[tauri::command]
fn profiles_save(file: Value) -> Result<Value, String> {
    refuse_if_root()?;
    let parsed: profiles::ProfilesFile = serde_json::from_value(file).map_err(|e| e.to_string())?;
    profiles::save(&profiles::config_dir(), &parsed)?;
    serde_json::to_value(parsed).map_err(|e| e.to_string())
}

#[tauri::command]
fn lighting_engine_install() -> Result<Value, String> {
    refuse_if_root()?;
    let path = openrgb_engine::install()?;
    Ok(serde_json::json!({ "ok": true, "path": path.display().to_string() }))
}

#[tauri::command]
fn lighting_hid_serial(path: String) -> Result<Value, String> {
    refuse_if_root()?;
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| n.starts_with("hidraw") && n.bytes().skip(6).all(|b| b.is_ascii_digit()))
        .ok_or_else(|| "invalid hidraw path".to_string())?;
    let serial = lighting::hid_serial(&lighting::hidraw_sys_root(), name);
    Ok(serde_json::json!({ "serial": serial }))
}

#[tauri::command]
fn update_check() -> Result<Value, String> {
    refuse_if_root()?;
    Ok(update_fetch::check_now())
}

#[tauri::command]
fn update_install() -> Result<Value, String> {
    refuse_if_root()?;
    update_fetch::install_now()
}

#[tauri::command]
fn open_url(url: String) -> Result<Value, String> {
    refuse_if_root()?;
    if !url.starts_with("https://github.com/edwardlthompson/chromaflow/") {
        return Err("refusing URL host".into());
    }
    Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
fn pwm_takeover(file: pwm_curves::CurveFile) -> Result<Value, String> {
    refuse_if_root()?;
    pwm_curves::save(&file)?;
    let detail = pwm_apply::tick(&collect_inventory(), &file)?;
    let daemon = pwm_daemon::enable().unwrap_or_else(|e| e);
    Ok(serde_json::json!({ "ok": true, "detail": format!("{detail}; {daemon}") }))
}

#[tauri::command]
fn pwm_tick() -> Result<Value, String> {
    refuse_if_root()?;
    let file = pwm_curves::load()?;
    let detail = pwm_apply::tick(&collect_inventory(), &file)?;
    Ok(serde_json::json!({ "ok": true, "detail": detail }))
}

#[tauri::command]
fn pwm_release() -> Result<Value, String> {
    refuse_if_root()?;
    let _ = pwm_daemon::disable();
    let restored = pwm_apply::failsafe()?;
    Ok(serde_json::json!({ "ok": true, "restored": restored }))
}

#[tauri::command]
fn pwm_load() -> Result<Value, String> {
    refuse_if_root()?;
    serde_json::to_value(pwm_curves::load()?).map_err(|e| e.to_string())
}

#[tauri::command]
fn pwm_store(file: pwm_curves::CurveFile) -> Result<Value, String> {
    refuse_if_root()?;
    pwm_curves::save(&file)?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
fn pwm_calibrate(dir: String, pwm: String) -> Result<Value, String> {
    refuse_if_root()?;
    let inv = collect_inventory();
    let mut file = pwm_curves::load()?;
    let rows = pwm_cal::sweep(&inv, &dir, &pwm, file.allow_zero)?;
    let key = format!("{dir}/{pwm}");
    if rows.last().map(|r| r[1]).unwrap_or(0) == 0 {
        file.calibration.insert(key, Vec::new());
    } else {
        file.calibration.insert(key, rows.clone());
    }
    pwm_curves::save(&file)?;
    let detail = pwm_apply::tick(&inv, &file).unwrap_or_else(|e| e);
    Ok(serde_json::json!({ "ok": true, "rows": rows, "detail": detail }))
}

fn prepare_webkit() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    // NVIDIA + Cinnamon: DMA-BUF off still leaves a blank #121418 surface.
    if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
}

fn main() {
    if let Err(err) = refuse_if_root() {
        eprintln!("{err}");
        std::process::exit(1);
    }
    prepare_webkit();
    let Some(lock) = single_instance::bind() else {
        std::process::exit(0);
    };
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            support_dry_run,
            support_apply,
            competitors_plan,
            competitors_remove,
            update_check,
            update_install,
            inventory,
            hardware_gauges,
            lighting_preview,
            lighting_sync,
            lighting_apply,
            lighting_broadcast,
            lighting_cycle,
            lighting_engine_install,
            lighting_hid_serial,
            open_url,
            profiles_load,
            profiles_save,
            session_load,
            session_save,
            pwm_takeover,
            pwm_tick,
            pwm_release,
            pwm_load,
            pwm_store,
            pwm_calibrate
        ])
        .setup(move |app| {
            single_instance::watch(app.handle().clone(), lock);
            icon::apply(app);
            let _ = tray::attach(app);
            window_geom::restore(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            window_geom::on_event(window, event);
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if !tray::quitting() {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }
            }
            if matches!(event, tauri::WindowEvent::Destroyed) && !pwm_daemon::is_active() {
                let _ = pwm_apply::failsafe();
            }
        })
        .run(tauri::generate_context!())
        .expect("chromaflow-gui failed to start");
}
