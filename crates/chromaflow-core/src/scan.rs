use crate::conflicts;
use crate::gaps;
use crate::hwmon;
use crate::nodes;
use crate::probes;
use crate::types::Inventory;

pub fn collect_inventory() -> Inventory {
    let hwmon_root = hwmon::hwmon_root();
    let dev_root = nodes::dev_root();
    let mut inv = Inventory {
        hwmon: hwmon::scan(&hwmon_root),
        hidraw: nodes::scan_prefix(&dev_root, "hidraw"),
        i2c: nodes::scan_prefix(&dev_root, "i2c-"),
        liquidctl: probes::liquidctl(),
        openrgb: probes::openrgb(),
        gaps: Vec::new(),
        conflicts: conflicts::detect(),
    };
    inv.gaps = gaps::from_inventory(&inv);
    inv
}
