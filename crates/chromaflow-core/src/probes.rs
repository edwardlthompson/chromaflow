use crate::types::BinaryProbe;
use std::process::Command;

pub fn liquidctl() -> BinaryProbe {
    match Command::new("liquidctl").arg("--version").output() {
        Ok(out) if out.status.success() => BinaryProbe {
            available: true,
            detail: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        },
        Ok(out) => BinaryProbe {
            available: false,
            detail: format!("exit {}", out.status),
        },
        Err(err) => BinaryProbe {
            available: false,
            detail: err.to_string(),
        },
    }
}

pub fn openrgb() -> crate::types::OpenRgbProbe {
    crate::openrgb::probe()
}
