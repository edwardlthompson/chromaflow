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

/// Drive one owned header to `target_pct`; all other owned headers stay at `floor`
/// so Auto-calibrate is audible one fan/pump at a time.
fn apply_solo(
    root: &Path,
    owned: &Path,
    dir: &str,
    pwm: &str,
    target_pct: u8,
    floor: u8,
    allow_zero: bool,
) -> Result<(), String> {
    let text = fs::read_to_string(owned).map_err(|e| e.to_string())?;
    let mut hit = false;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(d), Some(p)) = (parts.next(), parts.next()) else {
            continue;
        };
        if !pwm_policy::valid_dir(d) || !pwm_policy::valid_pwm(p) {
            continue;
        }
        let pct = if d == dir && p == pwm {
            hit = true;
            target_pct
        } else {
            floor
        };
        let duty = pwm_policy::percent_to_duty(pct, allow_zero)?;
        let path = root.join(d).join(p);
        pwm_apply::apply_enable(
            &path.parent().unwrap().join(format!("{p}_enable")),
            ENABLE_MANUAL,
        )?;
        pwm_apply::apply_duty(&path, duty, allow_zero)?;
    }
    if !hit {
        return Err("take-over first".into());
    }
    Ok(())
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
    let floor = if allow_zero {
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
            apply_solo(root, owned, dir, pwm, pct, floor, allow_zero)?;
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

fn hold_ms() -> u64 {
    std::env::var("CHROMAFLOW_IDENTIFY_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(if cfg!(test) { 0 } else { 5000 })
}

/// Spin one owned header at 100%; peers at floor (never silent 0% unless allow_zero).
/// Restores the saved recipe via `pwm_apply::tick` when done.
pub fn identify(inv: &Inventory, dir: &str, pwm: &str) -> Result<String, String> {
    let owned = pwm_apply::owned_path();
    let file = crate::pwm_curves::load().unwrap_or_else(|_| crate::pwm_curves::empty());
    let allow_zero = file.allow_zero;
    let floor = if allow_zero {
        0u8
    } else {
        file.min_duty.max(pwm_policy::MIN_PERCENT)
    };
    identify_at(
        &hwmon::hwmon_root(),
        &owned,
        &inv.conflicts,
        dir,
        pwm,
        hold_ms(),
        floor,
        allow_zero,
    )?;
    let detail = pwm_apply::tick(inv, &file).unwrap_or_else(|e| e);
    Ok(format!("identified {dir}/{pwm}; {detail}"))
}

#[allow(clippy::too_many_arguments)]
pub fn identify_at(
    root: &Path,
    owned: &Path,
    conflicts: &[String],
    dir: &str,
    pwm: &str,
    hold: u64,
    floor: u8,
    allow_zero: bool,
) -> Result<u32, String> {
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
    let result = (|| -> Result<u32, String> {
        let text = fs::read_to_string(owned).map_err(|e| e.to_string())?;
        let mut n = 0u32;
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let (Some(d), Some(p)) = (parts.next(), parts.next()) else {
                continue;
            };
            if !pwm_policy::valid_dir(d) || !pwm_policy::valid_pwm(p) {
                continue;
            }
            let pct = if d == dir && p == pwm { 100u8 } else { floor };
            let duty = pwm_policy::percent_to_duty(pct, allow_zero)?;
            let path = root.join(d).join(p);
            pwm_apply::apply_enable(
                &path.parent().unwrap().join(format!("{p}_enable")),
                ENABLE_MANUAL,
            )?;
            pwm_apply::apply_duty(&path, duty, allow_zero)?;
            n += 1;
        }
        if n == 0 {
            return Err("no owned channels".into());
        }
        if hold > 0 {
            thread::sleep(Duration::from_millis(hold));
        }
        Ok(n)
    })();
    if result.is_err() {
        let _ = pwm_apply::failsafe_at(root, owned);
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

    #[test]
    fn sweep_floors_peers_while_target_steps() {
        let root = std::env::temp_dir().join(format!("cf-cal-peer-{}", std::process::id()));
        let chip = root.join("hwmon0");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&chip).unwrap();
        for n in 1..=2u8 {
            fs::write(chip.join(format!("pwm{n}")), "200\n").unwrap();
            fs::write(chip.join(format!("pwm{n}_enable")), "2\n").unwrap();
            fs::write(chip.join(format!("fan{n}_input")), "900\n").unwrap();
        }
        let owned = root.join("owned");
        fs::write(&owned, "hwmon0 pwm1\nhwmon0 pwm2\n").unwrap();
        let rows = sweep_at(&root, &owned, &[], "hwmon0", "pwm1", false).unwrap();
        assert_eq!(rows.last().map(|r| r[0]), Some(100));
        assert_eq!(fs::read_to_string(chip.join("pwm1")).unwrap().trim(), "255");
        assert_eq!(
            fs::read_to_string(chip.join("pwm2")).unwrap().trim(),
            "51",
            "peer stays at 20% floor while target sweeps"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn identify_spins_target_and_floors_peer() {
        let root = std::env::temp_dir().join(format!("cf-id-{}", std::process::id()));
        let chip = root.join("hwmon0");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&chip).unwrap();
        fs::write(chip.join("pwm1"), "51\n").unwrap();
        fs::write(chip.join("pwm1_enable"), "2\n").unwrap();
        fs::write(chip.join("pwm2"), "51\n").unwrap();
        fs::write(chip.join("pwm2_enable"), "2\n").unwrap();
        let owned = root.join("owned");
        fs::write(&owned, "hwmon0 pwm1\nhwmon0 pwm2\n").unwrap();
        let n = identify_at(
            &root,
            &owned,
            &[],
            "hwmon0",
            "pwm1",
            0,
            pwm_policy::MIN_PERCENT,
            false,
        )
        .unwrap();
        assert_eq!(n, 2);
        assert_eq!(fs::read_to_string(chip.join("pwm1")).unwrap().trim(), "255");
        assert_eq!(fs::read_to_string(chip.join("pwm2")).unwrap().trim(), "51");
        let _ = fs::remove_dir_all(&root);
    }
}
