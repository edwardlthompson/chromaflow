//! Duty% to RPM sweep. Never silent 0%. Never enable 0.

use crate::hwmon;
use crate::pwm_apply;
use crate::pwm_hyst;
use crate::pwm_policy::{self, ENABLE_MANUAL};
use crate::types::Inventory;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

fn dwell_ms() -> u64 {
    std::env::var("CHROMAFLOW_CALIBRATE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(if cfg!(test) { 0 } else { 2000 })
}

fn lock_from(owned: &Path) -> PathBuf {
    owned.with_file_name("calibrating")
}

pub fn busy() -> bool {
    if pwm_hyst::paused() {
        return true;
    }
    lock_fresh(&lock_from(&pwm_apply::owned_path()))
}

fn lock_fresh(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return true;
    };
    modified.elapsed().map(|d| d.as_secs() < 90).unwrap_or(true)
}

fn set_busy_at(owned: &Path, v: bool) {
    pwm_hyst::set_paused(v);
    let path = lock_from(owned);
    if v {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&path, "1\n");
    } else {
        let _ = fs::remove_file(&path);
    }
}

fn fan_name(pwm: &str) -> Option<String> {
    pwm.strip_prefix("pwm")
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map(|n| format!("fan{n}_input"))
}

fn read_rpm(root: &Path, dir: &str, pwm: &str) -> u32 {
    let Some(name) = fan_name(pwm) else {
        return 0;
    };
    fs::read_to_string(root.join(dir).join(name))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn is_owned(owned: &Path, dir: &str, pwm: &str) -> bool {
    fs::read_to_string(owned)
        .ok()
        .map(|text| {
            text.lines().any(|line| {
                let mut parts = line.split_whitespace();
                parts.next() == Some(dir) && parts.next() == Some(pwm)
            })
        })
        .unwrap_or(false)
}

pub fn sweep(
    inv: &Inventory,
    dir: &str,
    pwm: &str,
    allow_zero: bool,
) -> Result<Vec<[u32; 2]>, String> {
    sweep_at(
        &hwmon::hwmon_root(),
        &pwm_apply::owned_path(),
        &inv.conflicts,
        dir,
        pwm,
        allow_zero,
    )
}

pub fn sweep_at(
    root: &Path,
    owned: &Path,
    conflicts: &[String],
    dir: &str,
    pwm: &str,
    allow_zero: bool,
) -> Result<Vec<[u32; 2]>, String> {
    if !conflicts.is_empty() {
        return Err(format!("conflicts: {}", conflicts.join(", ")));
    }
    if !pwm_policy::valid_dir(dir) || !pwm_policy::valid_pwm(pwm) {
        return Err("pwm path".into());
    }
    if !is_owned(owned, dir, pwm) {
        return Err("take-over first".into());
    }
    set_busy_at(owned, true);
    let start = if allow_zero {
        0u8
    } else {
        pwm_policy::MIN_PERCENT
    };
    let result = (|| {
        let mut rows = Vec::new();
        for pct in (start..=100).step_by(10) {
            if pct == 0 && !allow_zero {
                continue;
            }
            let duty = pwm_policy::percent_to_duty(pct, allow_zero)?;
            let path = root.join(dir).join(pwm);
            pwm_apply::apply_enable(
                &path.parent().unwrap().join(format!("{pwm}_enable")),
                ENABLE_MANUAL,
            )?;
            pwm_apply::apply_duty(&path, duty, allow_zero)?;
            let ms = dwell_ms();
            if ms > 0 {
                thread::sleep(Duration::from_millis(ms));
            }
            rows.push([u32::from(pct), read_rpm(root, dir, pwm)]);
        }
        Ok(rows)
    })();
    if result.is_err() {
        let _ = pwm_apply::restore_dir(root, dir, pwm);
    }
    set_busy_at(owned, false);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_skips_zero_and_keeps_zero_rpm() {
        let root = std::env::temp_dir().join(format!("cf-cal-{}", std::process::id()));
        let chip = root.join("hwmon0");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&chip).unwrap();
        fs::write(chip.join("pwm1"), "128\n").unwrap();
        fs::write(chip.join("pwm1_enable"), "2\n").unwrap();
        fs::write(chip.join("fan1_input"), "0\n").unwrap();
        let owned = root.join("owned");
        fs::write(&owned, "hwmon0 pwm1\n").unwrap();
        let rows = sweep_at(&root, &owned, &[], "hwmon0", "pwm1", false).unwrap();
        assert!(rows.iter().all(|r| r[0] != 0));
        assert!(rows.iter().all(|r| r[1] == 0));
        assert_ne!(fs::read_to_string(chip.join("pwm1")).unwrap().trim(), "0");
        assert!(pwm_policy::percent_to_duty(0, false).is_err());
        let _ = fs::remove_dir_all(&root);
    }
}
