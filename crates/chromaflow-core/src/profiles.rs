//! Profile store: curve set + RGB name. Never writes PWM.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfilesFile {
    pub schema: u32,
    pub active: Option<String>,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub curve_set: String,
    pub rgb: String,
}

pub fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_CONFIG_HOME") {
        return PathBuf::from(p);
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
        });
    base.join("chromaflow")
}

pub fn empty() -> ProfilesFile {
    ProfilesFile {
        schema: 1,
        active: None,
        profiles: Vec::new(),
    }
}

pub fn load(dir: &Path) -> Result<ProfilesFile, String> {
    let path = dir.join("profiles.json");
    if !path.is_file() {
        return Ok(empty());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file: ProfilesFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    validate(&file)?;
    Ok(file)
}

pub fn save(dir: &Path, file: &ProfilesFile) -> Result<(), String> {
    validate(file)?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join("profiles.json");
    let tmp = dir.join("profiles.json.tmp");
    let body = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    fs::write(&tmp, body).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

fn valid_token(s: &str) -> bool {
    let n = s.len();
    n >= 1
        && n <= 40
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn validate(file: &ProfilesFile) -> Result<(), String> {
    if file.schema != 1 {
        return Err("unsupported profiles schema".into());
    }
    if let Some(active) = &file.active {
        if !file.profiles.iter().any(|p| &p.name == active) {
            return Err("active profile is not in the list".into());
        }
    }
    for p in &file.profiles {
        if !valid_token(&p.name) || !valid_token(&p.curve_set) || !valid_token(&p.rgb) {
            return Err("profile fields must be [A-Za-z0-9_-]{1,40}".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn round_trip() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("chromaflow-profiles-{nanos}"));
        let file = ProfilesFile {
            schema: 1,
            active: Some("quiet".into()),
            profiles: vec![Profile {
                name: "quiet".into(),
                curve_set: "default".into(),
                rgb: "off".into(),
            }],
        };
        save(&dir, &file).unwrap();
        assert_eq!(load(&dir).unwrap(), file);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_bad_name() {
        let file = ProfilesFile {
            schema: 1,
            active: None,
            profiles: vec![Profile {
                name: "../x".into(),
                curve_set: "default".into(),
                rgb: "off".into(),
            }],
        };
        assert!(validate(&file).is_err());
    }
}
