//! Detect Flatpak/bwrap OpenRGB. No HID or PWM writes.

pub fn sandboxed() -> bool {
    if let Ok(raw) = std::env::var("CHROMAFLOW_OPENRGB_CMDLINE") {
        return raw.contains("bwrap");
    }
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return false;
    };
    dir.flatten().any(|ent| {
        let pid = ent.file_name();
        if !pid.to_string_lossy().bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        std::fs::read(ent.path().join("cmdline"))
            .ok()
            .map(|c| {
                let s = String::from_utf8_lossy(&c);
                s.contains("openrgb") && s.contains("bwrap")
            })
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn sandbox_env_override() {
        std::env::set_var("CHROMAFLOW_OPENRGB_CMDLINE", "bwrap -- openrgb --server");
        assert!(super::sandboxed());
        std::env::set_var("CHROMAFLOW_OPENRGB_CMDLINE", "openrgb --server");
        assert!(!super::sandboxed());
        std::env::remove_var("CHROMAFLOW_OPENRGB_CMDLINE");
    }
}
