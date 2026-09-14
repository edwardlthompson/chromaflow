//! Apply PWM under a hwmon root. Owned channels only. Never enable 0.

use crate::gauges;
use crate::hwmon;
use crate::pwm_curves::{Channel, CurveFile};
use crate::pwm_hyst;
use crate::pwm_policy::{self, ENABLE_FIRMWARE, ENABLE_MANUAL};
use crate::pwm_recipe;
use crate::types::Inventory;
use std::fs;
use std::path::{Path, PathBuf};

pub fn owned_path() -> PathBuf {
    if let Ok(p) = std::env::var("CHROMAFLOW_OWNED") {
        return PathBuf::from(p);
    }
    let base = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("chromaflow/owned")
}

fn pwm_path(root: &Path, dir: &str, name: &str) -> Result<PathBuf, String> {
    if !pwm_policy::valid_dir(dir) || !pwm_policy::valid_pwm(name) {
        return Err("pwm path".into());
    }
    let chip = root.join(dir);
    let chip_c = chip.canonicalize().unwrap_or(chip);
    let path = chip_c.join(name);
    let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
    if !canon.starts_with(&chip_c) {
        return Err("pwm path escapes hwmon chip".into());
    }
    Ok(canon)
}

fn put_u8(path: &Path, value: u8) -> Result<(), String> {
    fs::write(path, format!("{value}\n")).map_err(|e| e.to_string())
}

pub fn apply_enable(path: &Path, mode: u8) -> Result<(), String> {
    if mode != ENABLE_MANUAL && mode != ENABLE_FIRMWARE {
        return Err("pwm enable must be 1 or 2".into());
    }
    put_u8(path, mode)
}

pub fn apply_duty(path: &Path, duty: u8, allow_zero: bool) -> Result<(), String> {
    if duty == 0 && !allow_zero {
        return Err("0% needs an explicit confirm".into());
    }
    put_u8(path, duty)
}

pub fn restore_dir(root: &Path, dir: &str, pwm: &str) -> Result<(), String> {
    let path = pwm_path(root, dir, pwm)?;
    apply_enable(
        &path.parent().unwrap().join(format!("{pwm}_enable")),
        ENABLE_FIRMWARE,
    )
}

pub fn record_owned(channels: &[Channel]) -> Result<(), String> {
    let path = owned_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut lines = String::new();
    for ch in channels.iter().filter(|c| c.enabled) {
        if pwm_policy::valid_dir(&ch.dir) && pwm_policy::valid_pwm(&ch.pwm) {
            lines.push_str(&format!("{} {}\n", ch.dir, ch.pwm));
        }
    }
    fs::write(&path, lines).map_err(|e| e.to_string())
}

pub fn failsafe_at(root: &Path, owned: &Path) -> Result<u32, String> {
    let Ok(text) = fs::read_to_string(owned) else {
        return Ok(0);
    };
    let mut n = 0;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(dir), Some(pwm)) = (parts.next(), parts.next()) else {
            continue;
        };
        restore_dir(root, dir, pwm)?;
        n += 1;
    }
    Ok(n)
}

pub fn failsafe() -> Result<u32, String> {
    let n = failsafe_at(&hwmon::hwmon_root(), &owned_path())?;
    Ok(n + crate::nvidia_fan_apply::failsafe())
}

pub fn tick(inv: &Inventory, file: &CurveFile) -> Result<String, String> {
    if pwm_hyst::paused() || crate::pwm_calibrate::busy() {
        return Ok("calibrating".into());
    }
    let hwmon_on = file
        .channels
        .iter()
        .any(|c| c.enabled && pwm_policy::valid_dir(&c.dir));
    if hwmon_on && !inv.conflicts.is_empty() {
        return Err(format!("conflicts: {}", inv.conflicts.join(", ")));
    }
    let root = hwmon::hwmon_root();
    record_owned(&file.channels)?;
    let g = gauges::snapshot(&inv.hwmon);
    let mut n = 0u32;
    for ch in file
        .channels
        .iter()
        .filter(|c| c.enabled && pwm_policy::valid_dir(&c.dir))
    {
        n += one(&root, inv, file, ch, &g)?;
    }
    let gpu = crate::nvidia_fan_apply::tick(inv, file).unwrap_or_else(|e| e);
    Ok(format!("applied {n}; {gpu}"))
}

fn one(
    root: &Path,
    inv: &Inventory,
    file: &CurveFile,
    ch: &Channel,
    g: &gauges::Gauges,
) -> Result<u32, String> {
    let identify = pwm_recipe::flat_full(&ch.curve_id, file);
    let Some(temp) = pwm_recipe::temp_of(inv, file, ch, g) else {
        if !identify {
            restore_dir(root, &ch.dir, &ch.pwm)?;
            return Ok(0);
        }
        apply_manual(root, ch, 100, file.allow_zero)?;
        return Ok(1);
    };
    if inv.hwmon.iter().all(|c| c.name != ch.chip) {
        restore_dir(root, &ch.dir, &ch.pwm)?;
        return Ok(0);
    }
    let pts = pwm_recipe::points_for(&ch.curve_id, file);
    let mut pct = if identify {
        100
    } else {
        pwm_recipe::interp(temp, &pts)
    };
    let off = i16::from(ch.offset);
    pct = u8::try_from((i16::from(pct) + off).clamp(0, 100)).unwrap_or(pct);
    let floor = if ch.min_pct > 0 {
        ch.min_pct
    } else {
        file.min_duty
    };
    pct = pct.max(floor);
    if ch.stop_pct > 0 && pct <= ch.stop_pct && !file.allow_zero {
        pct = floor;
    }
    if !identify {
        pct = pwm_hyst::clamp(
            &pwm_hyst::key(&ch.dir, &ch.pwm),
            pct,
            temp,
            ch.hysteresis_c,
            ch.step_up,
            ch.step_down,
            ch.response_ms,
        );
    }
    apply_manual(root, ch, pct, file.allow_zero)?;
    Ok(1)
}

fn apply_manual(root: &Path, ch: &Channel, pct: u8, allow_zero: bool) -> Result<(), String> {
    let duty = pwm_policy::percent_to_duty(pct, allow_zero)?;
    let pwm = pwm_path(root, &ch.dir, &ch.pwm)?;
    apply_enable(
        &pwm.parent().unwrap().join(format!("{}_enable", ch.pwm)),
        ENABLE_MANUAL,
    )?;
    apply_duty(&pwm, duty, allow_zero)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_then_failsafe_never_zero_enable() {
        let root = std::env::temp_dir().join(format!("cf-pwm-{}", std::process::id()));
        let chip = root.join("hwmon0");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&chip).unwrap();
        fs::write(chip.join("pwm1"), "128\n").unwrap();
        fs::write(chip.join("pwm1_enable"), "2\n").unwrap();
        apply_enable(&chip.join("pwm1_enable"), 1).unwrap();
        apply_duty(&chip.join("pwm1"), 51, false).unwrap();
        assert!(apply_duty(&chip.join("pwm1"), 0, false).is_err());
        assert!(apply_enable(&chip.join("pwm1_enable"), 0).is_err());
        let owned = root.join("owned");
        fs::write(&owned, "hwmon0 pwm1\n").unwrap();
        assert_eq!(failsafe_at(&root, &owned).unwrap(), 1);
        assert_eq!(
            fs::read_to_string(chip.join("pwm1_enable")).unwrap().trim(),
            "2"
        );
        assert_eq!(fs::read_to_string(chip.join("pwm1")).unwrap().trim(), "51");
        let _ = fs::remove_dir_all(&root);
    }
}
