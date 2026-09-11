//! Unprivileged Tauri 2 shell. Refuses root. Dry-run support only. No PWM writes.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chromaflow_core::refuse_if_root;
use chromaflow_core::support;
use serde_json::Value;
use std::path::PathBuf;

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

fn main() {
    if let Err(err) = refuse_if_root() {
        eprintln!("{err}");
        std::process::exit(1);
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![support_dry_run])
        .run(tauri::generate_context!())
        .expect("chromaflow-gui failed to start");
}
