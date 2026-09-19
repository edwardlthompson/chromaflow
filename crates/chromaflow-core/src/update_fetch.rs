//! Curl GitHub Releases and pkexec the pinned helper. No PWM.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{json, Value};

use crate::privilege::refuse_if_root;
use crate::update_check::{classify_release, deb_name, pinned_asset_url, sha_arg_ok, UpdateReport};

const API: &str = "https://api.github.com/repos/edwardlthompson/chromaflow/releases/latest";
const HELPER: &str = "/usr/libexec/chromaflow/install-update.sh";

pub fn check_now() -> Value {
    match probe() {
        Ok(report) => to_json(&report),
        Err(status) => json!({
            "status": status, "current": env!("CARGO_PKG_VERSION"),
            "latest": "", "asset_url": "", "sha256": "",
        }),
    }
}

pub fn install_now() -> Result<Value, String> {
    refuse_if_root()?;
    let report = probe().map_err(|status| status.to_string())?;
    if report.status != "available" {
        return Ok(to_json(&report));
    }
    if report.asset_url != pinned_asset_url(&report.latest) || !sha_arg_ok(&report.sha256) {
        return Err("refusing asset".into());
    }
    if !Path::new(HELPER).is_file() {
        return Ok(
            json!({"status": "helper_missing", "asset_url": report.asset_url, "latest": report.latest}),
        );
    }
    let dest = runtime_dir()?.join(deb_name(&report.latest));
    download(&report.asset_url, &dest)?;
    if file_sha(&dest)? != report.sha256 {
        let _ = fs::remove_file(&dest);
        return Err("digest mismatch".into());
    }
    let out = Command::new("pkexec")
        .arg(HELPER)
        .arg(&dest)
        .arg(&report.sha256)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(if err.trim().is_empty() {
            "install failed".into()
        } else {
            err.trim().to_string()
        });
    }
    let _ = fs::remove_file(&dest);
    Ok(json!({ "status": "installed", "latest": report.latest }))
}

fn probe() -> Result<UpdateReport, &'static str> {
    Ok(classify_release(
        &fetch_api()?,
        env!("CARGO_PKG_VERSION"),
        &arch(),
    ))
}

fn curl(args: &[&str]) -> Result<Output, String> {
    Command::new("curl")
        .args(args)
        .output()
        .map_err(|e| e.to_string())
}

fn fetch_api() -> Result<String, &'static str> {
    let out = curl(&[
        "--silent",
        "--show-error",
        "--proto",
        "=https",
        "--tlsv1.2",
        "--max-time",
        "10",
        "--max-filesize",
        "1048576",
        "-A",
        "ChromaFlow",
        "-w",
        "\n%{http_code}",
        API,
    ])
    .map_err(|_| "offline")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (body, code) = text.rsplit_once('\n').unwrap_or((text.as_ref(), "0"));
    match code.trim() {
        "200" => Ok(body.to_string()),
        "403" => Err("rate_limited"),
        _ => Err("offline"),
    }
}

fn arch() -> String {
    let Ok(out) = Command::new("uname").arg("-m").output() else {
        return String::new();
    };
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn set_mode(path: &Path, bits: u32) -> Result<(), String> {
    let mut perm = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perm.set_mode(bits);
    fs::set_permissions(path, perm).map_err(|e| e.to_string())
}

fn runtime_dir() -> Result<PathBuf, String> {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    let root = if !base.is_empty() {
        PathBuf::from(base)
    } else {
        let out = Command::new("id")
            .arg("-u")
            .output()
            .map_err(|e| e.to_string())?;
        let uid = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !uid.chars().all(|c| c.is_ascii_digit()) {
            return Err("uid".into());
        }
        PathBuf::from(format!("/run/user/{uid}"))
    };
    let dir = root.join("chromaflow-update");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    set_mode(&dir, 0o700)?;
    Ok(dir)
}

fn download(url: &str, dest: &Path) -> Result<(), String> {
    let status = Command::new("curl")
        .args([
            "-fL",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-time",
            "120",
            "--max-filesize",
            "209715200",
            "-A",
            "ChromaFlow",
            "-o",
        ])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        let _ = fs::remove_file(dest);
        return Err("download failed".into());
    }
    set_mode(dest, 0o600)
}

fn file_sha(path: &Path) -> Result<String, String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("sha256sum failed".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let hex = text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if sha_arg_ok(&hex) {
        Ok(hex)
    } else {
        Err("sha256sum failed".into())
    }
}

fn to_json(report: &UpdateReport) -> Value {
    json!({
        "status": report.status, "current": report.current, "latest": report.latest,
        "asset_url": report.asset_url, "sha256": report.sha256,
    })
}
