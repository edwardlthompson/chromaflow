//! PWM enable/duty policy. Never silent 0%. Never enable 0.

pub const ENABLE_MANUAL: u8 = 1;
pub const ENABLE_FIRMWARE: u8 = 2;
pub const MIN_PERCENT: u8 = 20;

pub fn failsafe_enable() -> u8 {
    ENABLE_FIRMWARE
}

pub fn forbids_silent_zero() -> bool {
    failsafe_enable() != 0
}

pub fn valid_pwm(name: &str) -> bool {
    let rest = match name.strip_prefix("pwm") {
        Some(rest) => rest,
        None => return false,
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

pub fn valid_dir(name: &str) -> bool {
    let rest = match name.strip_prefix("hwmon") {
        Some(rest) => rest,
        None => return false,
    };
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

pub fn percent_to_duty(pct: u8, allow_zero: bool) -> Result<u8, String> {
    if pct == 0 && !allow_zero {
        return Err("0% needs an explicit confirm".into());
    }
    if pct > 100 {
        return Err("duty percent".into());
    }
    Ok(((u16::from(pct) * 255) / 100) as u8)
}

pub fn tuned(min_duty: u8, max_duty: u8) -> [(f32, u8); 4] {
    let lo = min_duty.min(100);
    let hi = max_duty.max(lo).min(100);
    let span = u16::from(hi.saturating_sub(lo));
    [
        (30.0, lo),
        (50.0, lo.saturating_add((span * 3 / 10) as u8)),
        (70.0, lo.saturating_add((span * 65 / 100) as u8)),
        (85.0, hi),
    ]
}

pub fn interpolate(temp_c: f32, min_duty: u8, max_duty: u8) -> u8 {
    let pts = tuned(min_duty, max_duty);
    if !temp_c.is_finite() {
        return min_duty.max(MIN_PERCENT);
    }
    if temp_c <= pts[0].0 {
        return pts[0].1;
    }
    if temp_c >= pts[3].0 {
        return pts[3].1;
    }
    for pair in pts.windows(2) {
        if temp_c <= pair[1].0 {
            let span = pair[1].0 - pair[0].0;
            let t = if span <= 0.0 {
                0.0
            } else {
                (temp_c - pair[0].0) / span
            };
            let a = f32::from(pair[0].1);
            let b = f32::from(pair[1].1);
            return (a + (b - a) * t).round() as u8;
        }
    }
    pts[3].1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failsafe_is_firmware_not_zero() {
        assert_eq!(failsafe_enable(), 2);
        assert!(forbids_silent_zero());
        assert!(valid_pwm("pwm1") && valid_dir("hwmon4"));
        assert!(!valid_pwm("pwm1_enable") && !valid_dir("it87952"));
        assert!(percent_to_duty(0, false).is_err());
        assert_eq!(percent_to_duty(0, true).unwrap(), 0);
        assert_eq!(percent_to_duty(20, false).unwrap(), 51);
        assert_eq!(interpolate(30.0, 20, 100), 20);
        assert_eq!(interpolate(85.0, 20, 100), 100);
    }
}
