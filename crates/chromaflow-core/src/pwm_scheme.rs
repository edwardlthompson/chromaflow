//! First Balanced control scheme. Never silent 0%. Never writes sysfs.

use crate::pwm_curves::{Channel, FanUnit};
use crate::pwm_policy::{self, MIN_PERCENT};
use crate::types::{GpuFan, HwmonChip, Inventory, PwmNode};

fn dir_of(chip: &HwmonChip) -> String {
    chip.path.rsplit('/').next().unwrap_or("").into()
}

fn header(chip: &str, pwm: &str) -> String {
    let c = chip.to_ascii_lowercase();
    if c.contains("it87952") {
        return match pwm {
            "pwm1" => "FAN4".into(),
            "pwm2" => "FAN5_PUMP".into(),
            "pwm3" => "FAN6_PUMP".into(),
            "pwm4" => "FAN7_PUMP".into(),
            "pwm5" => "FAN8".into(),
            _ => format!("{chip} {pwm}"),
        };
    }
    if c.contains("it8689") {
        return match pwm {
            "pwm1" => "CPU_FAN".into(),
            "pwm2" => "SYS_FAN1".into(),
            "pwm3" => "SYS_FAN2".into(),
            "pwm4" => "SYS_FAN3".into(),
            "pwm5" => "CPU_OPT".into(),
            _ => format!("{chip} {pwm}"),
        };
    }
    format!("{chip} {pwm}")
}

fn kind_of(name: &str) -> String {
    if name.to_ascii_uppercase().contains("PUMP") {
        "pump".into()
    } else {
        "fan".into()
    }
}

fn channel(chip: &str, pwm: &str, dir: &str, kind: &str, temp_id: &str) -> Channel {
    Channel {
        chip: chip.into(),
        pwm: pwm.into(),
        dir: dir.into(),
        enabled: true,
        curve_id: "balanced".into(),
        temp_id: temp_id.into(),
        kind: kind.into(),
        step_up: 5,
        step_down: 5,
        start_pct: MIN_PERCENT,
        min_pct: MIN_PERCENT,
        hysteresis_c: 3,
        ..Channel::default()
    }
}

fn writable(p: &PwmNode) -> bool {
    p.writable && p.enable_exists && pwm_policy::valid_pwm(&p.name)
}

pub fn balanced(inv: &Inventory) -> crate::pwm_curves::CurveFile {
    from_parts(&inv.hwmon, &inv.gpu_fans, &inv.conflicts)
}

pub fn from_parts(
    chips: &[HwmonChip],
    gpu: &[GpuFan],
    conflicts: &[String],
) -> crate::pwm_curves::CurveFile {
    let mut file = crate::pwm_curves::empty();
    file.min_duty = MIN_PERCENT;
    if conflicts.is_empty() {
        for chip in chips {
            let dir = dir_of(chip);
            if !pwm_policy::valid_dir(&dir) {
                continue;
            }
            for pwm in chip.pwms.iter().filter(|p| writable(p)) {
                let name = header(&chip.name, &pwm.name);
                let kind = kind_of(&name);
                let id = format!("{}-{}", chip.path, pwm.name);
                file.names.insert(id, name);
                file.channels
                    .push(channel(&chip.name, &pwm.name, &dir, &kind, "gauge:cpu"));
            }
        }
    }
    for fan in gpu.iter().filter(|f| f.writable) {
        let idx = fan.id.rsplit(':').next().unwrap_or("0");
        let pwm = format!("fan{idx}");
        file.names.insert(fan.id.clone(), fan.label.clone());
        file.channels
            .push(channel("nvidia", &pwm, "nvidia", "fan", "gauge:gpu"));
    }
    let members: Vec<String> = file
        .names
        .iter()
        .filter(|(_, n)| {
            n.to_ascii_uppercase().contains("PUMP") || n.to_ascii_lowercase().contains("gpu fan")
        })
        .map(|(id, _)| id.clone())
        .collect();
    if !members.is_empty() {
        file.units.push(FanUnit {
            id: "aio".into(),
            label: "AIO".into(),
            members,
        });
    }
    file
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LabeledValue;

    #[test]
    fn balanced_skips_hwmon_when_conflicts() {
        let chip = HwmonChip {
            name: "it8689".into(),
            path: "/sys/class/hwmon/hwmon4".into(),
            temps: vec![LabeledValue {
                label: "temp1".into(),
                value: "40000".into(),
            }],
            fans: Vec::new(),
            pwms: vec![PwmNode {
                name: "pwm1".into(),
                value: "128".into(),
                enable_exists: true,
                writable: true,
            }],
        };
        let gpu = vec![GpuFan {
            id: "nvidia:fan:0".into(),
            label: "GPU fan 0".into(),
            percent: Some(30),
            rpm: Some(900),
            writable: true,
        }];
        let blocked = from_parts(&[chip.clone()], &gpu, &["fancontrol".into()]);
        assert!(blocked.channels.iter().all(|c| c.chip == "nvidia"));
        assert_eq!(blocked.channels[0].curve_id, "balanced");
        assert_eq!(blocked.channels[0].min_pct, MIN_PERCENT);
        let open = from_parts(&[chip], &gpu, &[]);
        assert!(open
            .channels
            .iter()
            .any(|c| c.pwm == "pwm1" && c.kind == "fan"));
        assert!(open
            .channels
            .iter()
            .any(|c| c.chip == "nvidia" && c.temp_id == "gauge:gpu"));
        assert_eq!(header("it87952", "pwm2"), "FAN5_PUMP");
        assert_eq!(header("it87952", "pwm4"), "FAN7_PUMP");
        assert_eq!(kind_of("FAN5_PUMP"), "pump");
    }
}
