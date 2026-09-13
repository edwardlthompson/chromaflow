//! Fusion color via liquidctl CLI. Never vendors rgb_fusion2.py.

use std::process::Command;

pub fn set_fusion(hex: &str) -> Result<String, String> {
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("color must be RRGGBB".into());
    }
    let bin = std::env::var("CHROMAFLOW_LIQUIDCTL").unwrap_or_else(|_| "liquidctl".into());
    let _ = Command::new("timeout")
        .args(["3", &bin, "-m", "Fusion", "initialize"])
        .output();
    let out = Command::new("timeout")
        .args(["3", &bin, "-m", "Fusion", "set", "sync", "color", "fixed", hex])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("liquidctl Fusion failed: {err}"));
    }
    Ok(format!("set Fusion {hex} via liquidctl"))
}

#[cfg(test)]
mod tests {
    use super::set_fusion;

    #[test]
    fn rejects_empty_hex() {
        assert!(set_fusion("").is_err());
        assert!(set_fusion("00").is_err());
    }
}
