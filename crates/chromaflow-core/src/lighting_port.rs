//! Claim native lighting backends. OpenRGB is opt-in. No PWM.

pub fn enabled() -> bool {
    std::env::var("CHROMAFLOW_NATIVE_RGB").ok().as_deref() != Some("0")
}

pub fn fusionish(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("fusion") || n.contains("aorus") || n.contains("gigabyte")
}

pub fn gpuish(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("suprim liquid") || (n.contains("4090") && n.contains("msi"))
}

pub fn pad_id(raw: &str) -> String {
    let t = raw.trim().to_ascii_lowercase();
    let t = t.strip_prefix("0x").unwrap_or(&t);
    format!("{t:0>4}")
}

pub fn claim(vid: &str, pid: &str) -> Option<&'static str> {
    if !enabled() {
        return None;
    }
    match (pad_id(vid).as_str(), pad_id(pid).as_str()) {
        ("1038", "1a00") => Some("arena"),
        ("1038", "1856") => Some("prime"),
        ("3434", _) => Some("keychron"),
        _ => None,
    }
}

pub fn rewrite(backend: &str, device: &str) -> String {
    if !enabled() {
        return if backend == "keychron" {
            "openrgb".into()
        } else {
            backend.into()
        };
    }
    let b = backend.to_ascii_lowercase();
    if b == "liquidctl" || fusionish(device) {
        return "liquidctl".into();
    }
    if b == "msi_gpu" || gpuish(device) {
        return "msi_gpu".into();
    }
    if b == "keychron" || skip_sdk(device) {
        return "keychron".into();
    }
    backend.into()
}

pub fn arenaish(name: &str) -> bool {
    name.to_ascii_lowercase().contains("arena")
}

pub fn skip_sdk(name: &str) -> bool {
    if !enabled() {
        return false;
    }
    let n = name.to_ascii_lowercase();
    n.contains("keychron") || n.contains("q6")
}

#[cfg(test)]
mod tests {
    #[test]
    fn claim_table_and_rewrite() {
        if !super::enabled() {
            return;
        }
        assert_eq!(super::claim("3434", "0b60"), Some("keychron"));
        assert_eq!(super::claim("1038", "1a00"), Some("arena"));
        assert_eq!(super::rewrite("openrgb", "Keychron Q6 HE"), "keychron");
        assert_eq!(super::rewrite("openrgb", "X570S AORUS MASTER"), "liquidctl");
        assert_eq!(
            super::rewrite("openrgb", "MSI GeForce RTX 4090 Suprim Liquid X"),
            "msi_gpu"
        );
        assert!(super::skip_sdk("Keychron Q6 HE"));
        assert!(super::arenaish("SteelSeries Arena 7"));
        assert_eq!(super::pad_id("0x3434"), "3434");
        assert_eq!(super::pad_id("1038"), "1038");
    }
}
