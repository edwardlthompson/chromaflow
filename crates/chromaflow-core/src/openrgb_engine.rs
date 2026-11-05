//! Pin + user-space download for the OpenRGB engine file. Never execs a .part.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::fs_mode;

#[derive(Debug, Deserialize)]
pub struct EnginePin {
    pub host: String,
    pub url: String,
    pub sha256: String,
    pub max_bytes: u64,
}

pub fn share_dir() -> PathBuf {
    crate::paths::data_dir()
}

pub fn pin_path() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_ENGINE_YAML") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("CHROMAFLOW_DATA") {
        return PathBuf::from(p).join("openrgb-engine.yaml");
    }
    #[cfg(windows)]
    {
        let packaged = PathBuf::from(r"C:\Program Files\ChromaFlow\openrgb-engine-windows.yaml");
        if packaged.is_file() {
            return packaged;
        }
        let win = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/openrgb-engine-windows.yaml");
        if win.is_file() {
            return win;
        }
    }
    let usr = PathBuf::from("/usr/share/chromaflow/openrgb-engine.yaml");
    if usr.is_file() {
        return usr;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/openrgb-engine.yaml")
}

pub fn load_pin() -> Result<EnginePin, String> {
    let path = pin_path();
    let raw = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| e.to_string())
}

pub fn host_allowed(url: &str, host: &str) -> bool {
    let rest = url.strip_prefix("https://").unwrap_or("");
    let got = rest.split('/').next().unwrap_or("");
    !host.is_empty() && got.eq_ignore_ascii_case(host)
}

pub fn dest_path() -> PathBuf {
    #[cfg(windows)]
    {
        // Private lighting engine — not a user-facing OpenRGB install.
        share_dir().join("engine").join("OpenRGB").join("OpenRGB.exe")
    }
    #[cfg(not(windows))]
    {
        share_dir().join("OpenRGB.AppImage")
    }
}

pub fn packaged_engine_path() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(r"C:\Program Files\ChromaFlow\engine\OpenRGB\OpenRGB.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/usr/libexec/chromaflow/OpenRGB.AppImage")
    }
}

pub fn start_user_unit() {
    #[cfg(windows)]
    {
        return;
    }
    #[cfg(not(windows))]
    {
        let quiet = |args: &[&str]| {
            let _ = Command::new("systemctl")
                .args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        };
        quiet(&["--user", "daemon-reload"]);
        if crate::openrgb_spawn::sdk_opt_in() {
            quiet(&["--user", "enable", "--now", "chromaflow-sdk.service"]);
        } else {
            quiet(&["--user", "disable", "--now", "chromaflow-sdk.service"]);
        }
    }
}

pub fn file_sha256(path: &Path) -> Result<String, String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("certutil");
        cmd.args(["-hashfile", &path.display().to_string(), "SHA256"]);
        let out = crate::win_cmd::hidden_capture(cmd)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("certutil sha256 failed".into());
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let hex = text
            .lines()
            .map(str::trim)
            .find(|l| l.chars().all(|c| c.is_ascii_hexdigit()) && l.len() == 64)
            .unwrap_or("")
            .to_ascii_lowercase();
        if hex.len() == 64 {
            Ok(hex)
        } else {
            Err("certutil sha256 failed".into())
        }
    }
    #[cfg(not(windows))]
    {
        let out = Command::new("sha256sum")
            .arg(path)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err("sha256sum failed".into());
        }
        let text = String::from_utf8_lossy(&out.stdout);
        Ok(text
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase())
    }
}

pub fn finalize_part(part: &Path, dest: &Path, sha: &str, max_bytes: u64) -> Result<(), String> {
    let meta = fs::metadata(part).map_err(|e| {
        let _ = fs::remove_file(part);
        e.to_string()
    })?;
    if meta.len() > max_bytes {
        let _ = fs::remove_file(part);
        return Err("engine download exceeds size cap".into());
    }
    let got = match file_sha256(part) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(part);
            return Err(e);
        }
    };
    if got != sha.to_ascii_lowercase() {
        let _ = fs::remove_file(part);
        return Err("engine sha256 mismatch".into());
    }
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::rename(part, dest).map_err(|e| {
        let _ = fs::remove_file(part);
        e.to_string()
    })?;
    fs_mode::set_mode(dest, 0o755)
}

pub fn install() -> Result<PathBuf, String> {
    let pin = load_pin()?;
    if !host_allowed(&pin.url, &pin.host) {
        return Err("engine URL host is not allowlisted".into());
    }
    let dest = dest_path();
    let part = {
        #[cfg(windows)]
        {
            dest.with_extension("exe.part")
        }
        #[cfg(not(windows))]
        {
            dest.with_extension("AppImage.part")
        }
    };
    let _ = fs::remove_file(&part);
    if let Some(dir) = part.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut curl = Command::new("curl");
    curl.args([
        "-fL",
        "--proto",
        "=https",
        "--tlsv1.2",
        "--max-time",
        "60",
        "--max-filesize",
        &pin.max_bytes.to_string(),
        "-o",
    ])
    .arg(&part)
    .arg(&pin.url);
    let status = crate::win_cmd::hidden(curl)
        .status()
        .map_err(|e| {
            let _ = fs::remove_file(&part);
            e.to_string()
        })?;
    if !status.success() {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "engine download failed (offline? save {} into {}).",
            pin.url,
            dest.display()
        ));
    }
    finalize_part(&part, &dest, &pin.sha256, pin.max_bytes)?;
    Ok(dest)
}

/// Copy a portable OpenRGB tree into the private ChromaFlow engine path (Windows).
/// Source is `CHROMAFLOW_OPENRGB` (path to OpenRGB.exe) or an already-resolved binary.
#[cfg(windows)]
pub fn seed_private_engine(from_exe: &Path) -> Result<PathBuf, String> {
    if !from_exe.is_file() {
        return Err(format!("source missing: {}", from_exe.display()));
    }
    let dest = dest_path();
    if dest.is_file() {
        return Ok(dest);
    }
    let src_dir = from_exe
        .parent()
        .ok_or_else(|| "OpenRGB.exe has no parent directory".to_string())?;
    let dest_dir = dest
        .parent()
        .ok_or_else(|| "engine dest has no parent".to_string())?;
    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    copy_dir_recursive(src_dir, dest_dir)?;
    if !dest.is_file() {
        return Err(format!(
            "copied {} but {} is missing",
            src_dir.display(),
            dest.display()
        ));
    }
    Ok(dest)
}

#[cfg(windows)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Quiet the private OpenRGB engine UI: no I2C warning, no "Resize the zones" prompt.
pub fn suppress_i2c_warning_dialogs() -> Result<(), String> {
    configure_engine_ui().map(|_| ())
}

/// Write OpenRGB.json so the private engine never prompts the user.
/// Returns `true` when the file changed (caller should restart the engine so settings load).
pub fn configure_engine_ui() -> Result<bool, String> {
    #[cfg(windows)]
    {
        // Hashes from MSVC std::hash<std::string> of OpenRGB's English WARNING HTML
        // (WinRing0 0.9 wording and PawnIO modern wording).
        const HASHES: &[u64] = &[5179838328490249627, 15643047387007252051];
        let path = openrgb_user_settings_path();
        let mut root: serde_json::Value = if path.is_file() {
            let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        let obj = root
            .as_object_mut()
            .ok_or_else(|| "OpenRGB.json root".to_string())?;
        let mut dirty = false;
        {
            let list = obj
                .entry("LogManager")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or_else(|| "LogManager".to_string())?
                .entry("dialog_no_show_hashes")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or_else(|| "hashes".to_string())?;
            for h in HASHES {
                let v = serde_json::json!(h);
                if !list.iter().any(|x| x == &v) {
                    list.push(v);
                    dirty = true;
                }
            }
        }
        {
            let ui = obj
                .entry("UserInterface")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or_else(|| "UserInterface".to_string())?;
            // OpenRGB 0.9 used PascalCase; 1.0+ uses snake_case. Write both.
            for key in ["run_zone_checks", "RunZoneChecks"] {
                if ui.get(key) != Some(&serde_json::json!(false)) {
                    dirty = true;
                }
                ui.insert(key.into(), serde_json::json!(false));
            }
        }
        if !dirty {
            return Ok(false);
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let out = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
        fs::write(&path, out).map_err(|e| e.to_string())?;
        Ok(true)
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

#[cfg(windows)]
fn openrgb_user_settings_path() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("OpenRGB")
        .join("OpenRGB.json")
}

#[cfg(windows)]
fn engine_reports_modern(exe: &Path) -> bool {
    // OpenRGB 0.9 is too old for Keychron Q6 HE / current zone settings keys.
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!(
            "(Get-Item -LiteralPath '{}').VersionInfo.ProductVersion",
            exe.display().to_string().replace('\'', "''")
        ),
    ]);
    let ver = crate::win_cmd::hidden_capture(cmd)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if ver.starts_with('1') || ver.contains("1.0") {
        return true;
    }
    // Fallback: PawnIO sibling ships with 1.0 portable trees, not 0.9.
    exe.parent()
        .map(|d| d.join("PawnIOLib.dll").is_file())
        .unwrap_or(false)
}

/// Ensure the private lighting engine exists under ChromaFlow paths.
pub fn ensure_private_engine() -> Result<String, String> {
    #[cfg(windows)]
    {
        let dest = dest_path();
        let marker = dest
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".engine-sha256");
        let pin_ok = load_pin()
            .ok()
            .map(|p| {
                fs::read_to_string(&marker)
                    .map(|s| s.trim() == p.sha256)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if dest.is_file() && pin_ok {
            return Ok(format!("private lighting engine at {}", dest.display()));
        }
        if packaged_engine_path().is_file() && pin_ok {
            return Ok(format!(
                "packaged lighting engine at {}",
                packaged_engine_path().display()
            ));
        }
        // Prefer an already-extracted modern tree (Support / agent may have staged 1.0).
        if dest.is_file() && engine_reports_modern(&dest) {
            if let Ok(pin) = load_pin() {
                let _ = fs::write(&marker, &pin.sha256);
            }
            return Ok(format!("private lighting engine at {}", dest.display()));
        }
        if let Ok(p) = std::env::var("CHROMAFLOW_OPENRGB") {
            let src = PathBuf::from(p);
            let got = seed_private_engine(&src)?;
            if let Ok(pin) = load_pin() {
                let _ = fs::write(
                    got.parent().unwrap_or_else(|| Path::new(".")).join(".engine-sha256"),
                    &pin.sha256,
                );
            }
            return Ok(format!("seeded private lighting engine at {}", got.display()));
        }
        Err(
            "Lighting engine not installed or outdated (need OpenRGB 1.0). Set CHROMAFLOW_OPENRGB or run Support after placing the portable zip."
                .into(),
        )
    }
    #[cfg(not(windows))]
    {
        let dest = dest_path();
        if dest.is_file() || packaged_engine_path().is_file() {
            Ok("lighting engine present".into())
        } else {
            Err("OpenRGB AppImage not installed yet".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn host_and_hash_reject() {
        assert!(host_allowed(
            "https://codeberg.org/OpenRGB/OpenRGB/releases/download/x.AppImage",
            "codeberg.org",
        ));
        assert!(!host_allowed("http://codeberg.org/x", "codeberg.org"));
        assert!(!host_allowed("https://evil.example/x", "codeberg.org"));
        let dir = std::env::temp_dir().join("cf-engine-hash");
        let _ = fs::create_dir_all(&dir);
        let part = dir.join("OpenRGB.AppImage.part");
        let dest = dir.join("OpenRGB.AppImage");
        let _ = fs::remove_file(&dest);
        let mut f = fs::File::create(&part).unwrap();
        f.write_all(b"not-openrgb").unwrap();
        drop(f);
        let err = finalize_part(&part, &dest, "deadbeef", 80).unwrap_err();
        assert!(err.contains("sha256"));
        assert!(!dest.exists());
        assert!(!part.exists());
    }
}
