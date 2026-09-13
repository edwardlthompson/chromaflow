use crate::types::{HwmonChip, LabeledValue, PwmNode};
use std::fs;
use std::path::{Path, PathBuf};

pub fn hwmon_root() -> PathBuf {
    std::env::var("CHROMAFLOW_HWMON_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/class/hwmon"))
}

pub fn scan(root: &Path) -> Vec<HwmonChip> {
    scan_kind(root, true)
}

pub fn scan_temps(root: &Path) -> Vec<HwmonChip> {
    scan_kind(root, false)
}

fn scan_kind(root: &Path, fans_pwm: bool) -> Vec<HwmonChip> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut chips = Vec::new();
    let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    dirs.sort();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let name = read_trimmed(&dir.join("name")).unwrap_or_else(|| "unknown".into());
        chips.push(HwmonChip {
            name,
            path: dir.display().to_string(),
            temps: labeled(&dir, "temp", "_input"),
            fans: if fans_pwm {
                labeled(&dir, "fan", "_input")
            } else {
                Vec::new()
            },
            pwms: if fans_pwm { pwms(&dir) } else { Vec::new() },
        });
    }
    chips
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn labeled(dir: &Path, prefix: &str, suffix: &str) -> Vec<LabeledValue> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in entries.flatten() {
        let fname = ent.file_name().to_string_lossy().into_owned();
        if fname.starts_with(prefix) && fname.ends_with(suffix) {
            if let Some(value) = read_trimmed(&ent.path()) {
                let label = fname
                    .strip_suffix(suffix)
                    .and_then(|stem| read_trimmed(&dir.join(format!("{stem}_label"))))
                    .filter(|s| !s.is_empty())
                    .unwrap_or(fname);
                out.push(LabeledValue { label, value });
            }
        }
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

fn pwms(dir: &Path) -> Vec<PwmNode> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for ent in entries.flatten() {
        let fname = ent.file_name().to_string_lossy().into_owned();
        if fname.starts_with("pwm") && !fname.contains('_') {
            names.push(fname);
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let path = dir.join(&name);
            // IT8795x pwm4/pwm5 read ENODATA until pwmN_enable=1; still list them.
            let value = read_trimmed(&path).filter(|v| !v.is_empty()).unwrap_or_else(|| "0".into());
            let enable = dir.join(format!("{name}_enable"));
            let writable = fs::metadata(&path)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false);
            PwmNode {
                name,
                value,
                enable_exists: enable.exists(),
                writable,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn fixture_chip() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/hwmon");
        let chips = scan(&root);
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].name, "nct6775");
        assert_eq!(chips[0].temps.len(), 1);
        assert_eq!(chips[0].pwms.len(), 1);
        assert!(chips[0].pwms[0].enable_exists);
        let temps = scan_temps(&root);
        assert_eq!(temps[0].temps.len(), 1);
        assert!(temps[0].pwms.is_empty());
        assert!(temps[0].fans.is_empty());
    }

    #[test]
    fn lists_pwm_when_duty_unreadable() {
        let root = std::env::temp_dir().join(format!("cf-hwmon-{}", std::process::id()));
        let chip = root.join("hwmon0");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&chip).unwrap();
        fs::write(chip.join("name"), "it87952\n").unwrap();
        fs::write(chip.join("pwm1"), "128\n").unwrap();
        fs::write(chip.join("pwm1_enable"), "2\n").unwrap();
        fs::write(chip.join("pwm4"), "").unwrap();
        fs::write(chip.join("pwm4_enable"), "2\n").unwrap();
        let chips = scan(&root);
        assert_eq!(chips[0].pwms.len(), 2);
        assert_eq!(chips[0].pwms[1].name, "pwm4");
        assert_eq!(chips[0].pwms[1].value, "0");
        assert!(chips[0].pwms[1].enable_exists);
        let _ = fs::remove_dir_all(&root);
    }
}
