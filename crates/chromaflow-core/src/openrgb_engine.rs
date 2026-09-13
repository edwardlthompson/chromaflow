//! Pin + user-space download for the OpenRGB engine file. Never execs a .part.

use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct EnginePin {
    pub host: String,
    pub url: String,
    pub sha256: String,
    pub max_bytes: u64,
}

pub fn share_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_DATA_HOME") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(p).join("chromaflow");
    }
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/share/chromaflow"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/chromaflow"))
}

pub fn pin_path() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_ENGINE_YAML") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("CHROMAFLOW_DATA") {
        return PathBuf::from(p).join("openrgb-engine.yaml");
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
    share_dir().join("OpenRGB.AppImage")
}

pub fn start_user_unit() {
    let quiet = |args: &[&str]| {
        let _ = Command::new("systemctl")
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    quiet(&["--user", "daemon-reload"]);
    quiet(&["--user", "enable", "--now", "chromaflow-sdk.service"]);
}

pub fn file_sha256(path: &Path) -> Result<String, String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("sha256sum failed".into());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text.split_whitespace().next().unwrap_or("").to_ascii_lowercase())
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
    let mut perm = fs::metadata(dest).map_err(|e| e.to_string())?.permissions();
    perm.set_mode(0o755);
    fs::set_permissions(dest, perm).map_err(|e| e.to_string())
}

pub fn install() -> Result<PathBuf, String> {
    let pin = load_pin()?;
    if !host_allowed(&pin.url, &pin.host) {
        return Err("engine URL host is not allowlisted".into());
    }
    let dest = dest_path();
    let part = dest.with_extension("AppImage.part");
    let _ = fs::remove_file(&part);
    if let Some(dir) = part.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let status = Command::new("curl")
        .args([
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
        .arg(&pin.url)
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
