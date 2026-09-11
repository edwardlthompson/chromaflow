use crate::names::{apt_name_ok, module_name_ok};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct IndexFile {
    packages: String,
    modules_safe: String,
    modules_experimental: String,
}

#[derive(Debug, Deserialize)]
struct ItemsFile {
    items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(default)]
    pub modprobe: Option<String>,
    #[serde(default)]
    pub apt: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub require_dpkg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Allowlist {
    pub packages: Vec<Item>,
    pub modules_safe: Vec<Item>,
    pub modules_experimental: Vec<Item>,
}

pub fn load(data_dir: &Path) -> Result<Allowlist, String> {
    let index: IndexFile = read_yaml(&data_dir.join("linux-support.yaml"))?;
    let packages = read_items(&data_dir.join(&index.packages))?;
    let modules_safe = read_items(&data_dir.join(&index.modules_safe))?;
    let modules_experimental = read_items(&data_dir.join(&index.modules_experimental))?;
    for item in packages
        .iter()
        .chain(&modules_safe)
        .chain(&modules_experimental)
    {
        if let Some(m) = item.modprobe.as_deref() {
            if !module_name_ok(m) {
                return Err(format!("illegal module name: {m}"));
            }
        }
        if let Some(a) = item.apt.as_deref() {
            if !apt_name_ok(a) {
                return Err(format!("illegal apt name: {a}"));
            }
        }
    }
    Ok(Allowlist {
        packages,
        modules_safe,
        modules_experimental,
    })
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_items(path: &Path) -> Result<Vec<Item>, String> {
    let file: ItemsFile = read_yaml(path)?;
    Ok(file.items)
}

pub fn contains_module(list: &Allowlist, name: &str) -> bool {
    list.modules_safe
        .iter()
        .chain(&list.modules_experimental)
        .any(|i| i.modprobe.as_deref() == Some(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn yaml_allowlist_loads() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let list = load(&dir).expect("allowlist");
        assert!(contains_module(&list, "nct6775"));
        assert!(!contains_module(&list, "not-a-real-module"));
        assert!(list
            .modules_experimental
            .iter()
            .any(|i| i.modprobe.as_deref() == Some("i2c-nvidia-gpu")));
    }
}
