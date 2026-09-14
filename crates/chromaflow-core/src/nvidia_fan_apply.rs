//! NVIDIA fan apply via nvidia-settings. Never silent 0%. No sysfs pwm*.

use crate::gauges;
use crate::pwm_curves::{Channel, CurveFile};
use crate::pwm_hyst;
use crate::pwm_policy;
use crate::pwm_recipe;
use crate::types::Inventory;
use std::collections::BTreeMap;
use std::sync::Mutex;

static LAST: Mutex<BTreeMap<u8, u8>> = Mutex::new(BTreeMap::new());

fn last_same(idx: u8, pct: u8) -> bool {
    LAST.lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&idx)
        .copied()
        == Some(pct)
}

fn remember(idx: u8, pct: u8) {
    LAST.lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(idx, pct);
}

fn clear_last() {
    LAST.lock().unwrap_or_else(|p| p.into_inner()).clear();
}

pub fn set_percent(idx: u8, pct: u8, allow_zero: bool) -> Result<(), String> {
    if pct == 0 && !allow_zero {
        return Err("0% needs an explicit confirm".into());
    }
    let floor = if allow_zero {
        0
    } else {
        pwm_policy::MIN_PERCENT
    };
    let pct = pct.max(floor).min(100);
    if last_same(idx, pct) {
        return Ok(());
    }
    let st = crate::nvidia_fans::command()
        .args([
            "-a",
            "GPUFanControlState=1",
            "-a",
            &format!("[fan:{idx}]/GPUTargetFanSpeed={pct}"),
        ])
        .status()
        .map_err(|e| e.to_string())?;
    if st.success() {
        remember(idx, pct);
        Ok(())
    } else {
        Err("nvidia-settings fan apply failed (Coolbits?)".into())
    }
}

pub fn failsafe() -> u32 {
    let on = crate::pwm_curves::load()
        .map(|f| f.channels.iter().any(|c| c.enabled && c.chip == "nvidia"))
        .unwrap_or(false);
    if !on {
        return 0;
    }
    clear_last();
    crate::nvidia_fans::command()
        .args(["-a", "GPUFanControlState=0"])
        .status()
        .map(|s| u32::from(s.success()))
        .unwrap_or(0)
}

pub fn tick(inv: &Inventory, file: &CurveFile) -> Result<String, String> {
    if pwm_hyst::paused() {
        return Ok("calibrating".into());
    }
    let g = gauges::snapshot(&inv.hwmon);
    let mut n = 0u32;
    for ch in file
        .channels
        .iter()
        .filter(|c| c.enabled && c.chip == "nvidia")
    {
        if one(inv, file, ch, &g) {
            n += 1;
        }
    }
    Ok(format!("gpu {n}"))
}

fn one(inv: &Inventory, file: &CurveFile, ch: &Channel, g: &gauges::Gauges) -> bool {
    let identify = pwm_recipe::flat_full(&ch.curve_id, file);
    let Some(temp) = pwm_recipe::temp_of(inv, file, ch, g) else {
        return identify && set_percent(fan_idx(ch), 100, file.allow_zero).is_ok();
    };
    let mut pct = if identify {
        100
    } else {
        pwm_recipe::interp(temp, &pwm_recipe::points_for(&ch.curve_id, file))
    };
    let floor = if ch.min_pct > 0 {
        ch.min_pct
    } else {
        file.min_duty
    };
    pct = pct.max(floor);
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
    let idx = fan_idx(ch);
    if last_same(idx, pct) {
        return false;
    }
    set_percent(idx, pct, file.allow_zero).is_ok()
}

fn fan_idx(ch: &Channel) -> u8 {
    ch.pwm
        .strip_prefix("fan")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn never_silent_zero() {
        assert!(super::set_percent(0, 0, false).is_err());
    }

    #[test]
    fn skip_repeat_percent() {
        super::clear_last();
        super::remember(0, 20);
        assert!(super::last_same(0, 20));
        assert!(!super::last_same(0, 30));
        super::clear_last();
        assert!(!super::last_same(0, 20));
    }
}
