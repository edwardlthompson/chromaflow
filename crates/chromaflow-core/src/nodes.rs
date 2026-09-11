use crate::types::DevNode;
use std::fs;
use std::path::{Path, PathBuf};

pub fn dev_root() -> PathBuf {
    std::env::var("CHROMAFLOW_DEV_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/dev"))
}

pub fn scan_prefix(root: &Path, prefix: &str) -> Vec<DevNode> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in entries.flatten() {
        let name = ent.file_name().to_string_lossy().into_owned();
        if !name.starts_with(prefix) {
            continue;
        }
        let path = ent.path();
        let readable = fs::File::open(&path).is_ok();
        out.push(DevNode {
            path: path.display().to_string(),
            readable,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}
