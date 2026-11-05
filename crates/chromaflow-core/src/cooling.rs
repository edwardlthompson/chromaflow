//! Platform cooling backends. Linux uses hwmon; Windows uses SuperIO via LPC helper.

use crate::types::HwmonChip;

/// Snapshot of controllable fans/temps for the Cooling UI.
#[derive(Debug, Clone, Default)]
pub struct CoolingSnapshot {
    pub chips: Vec<HwmonChip>,
    pub source: &'static str,
    pub gaps: Vec<String>,
}

pub trait CoolingBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn snapshot(&self) -> CoolingSnapshot;
    fn pwm_apply(&self, chip: &str, pwm: &str, duty: u8, allow_zero: bool) -> Result<(), String>;
    fn release(&self, chip: &str, pwm: &str) -> Result<(), String>;
}

pub fn active() -> Box<dyn CoolingBackend> {
    crate::platform::cooling_backend()
}

#[cfg(not(windows))]
pub struct LinuxHwmonBackend;

#[cfg(not(windows))]
impl CoolingBackend for LinuxHwmonBackend {
    fn name(&self) -> &'static str {
        "hwmon"
    }

    fn snapshot(&self) -> CoolingSnapshot {
        let root = crate::hwmon::hwmon_root();
        CoolingSnapshot {
            chips: crate::hwmon::scan(&root),
            source: "hwmon",
            gaps: Vec::new(),
        }
    }

    fn pwm_apply(&self, chip: &str, pwm: &str, duty: u8, allow_zero: bool) -> Result<(), String> {
        let root = crate::hwmon::hwmon_root();
        let path = root.join(chip).join(pwm);
        crate::pwm_apply::apply_duty(&path, duty, allow_zero)?;
        let enable = path.parent().unwrap().join(format!("{pwm}_enable"));
        crate::pwm_apply::apply_enable(&enable, crate::pwm_policy::ENABLE_MANUAL)
    }

    fn release(&self, chip: &str, pwm: &str) -> Result<(), String> {
        crate::pwm_apply::restore_dir(&crate::hwmon::hwmon_root(), chip, pwm)
    }
}

/// Merge platform chips into inventory-shaped hwmon list (Windows SuperIO appears as chips).
pub fn inventory_chips() -> Vec<HwmonChip> {
    let snap = active().snapshot();
    if !snap.chips.is_empty() {
        return snap.chips;
    }
    crate::hwmon::scan(&crate::hwmon::hwmon_root())
}
