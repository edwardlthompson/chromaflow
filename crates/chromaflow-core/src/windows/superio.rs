//! ITE / Nuvoton SuperIO inventory for Windows. Register maps are original Rust
//! (public chip docs); no LibreHardwareMonitor or Fan Control source.

use crate::cooling::{CoolingBackend, CoolingSnapshot};
use crate::hwmon;
use crate::types::{HwmonChip, LabeledValue, PwmNode};
use crate::windows::lpc::{self, IoKind};
use crate::windows::lpc_client;

/// Known SuperIO chip families we aim to control (acceptance: Gigabyte dual-ITE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipFamily {
    It87,
    Nct6775,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SuperIoChip {
    pub family: ChipFamily,
    pub name: String,
    pub chip_id: u16,
    pub pwm_count: u8,
    pub fan_count: u8,
}

#[derive(Default)]
pub struct WindowsSuperIoBackend;

impl WindowsSuperIoBackend {
    fn probe(&self) -> Vec<SuperIoChip> {
        match lpc::kind() {
            IoKind::Mock => vec![
                SuperIoChip {
                    family: ChipFamily::It87,
                    name: "it8689".into(),
                    chip_id: 0x8689,
                    pwm_count: 5,
                    fan_count: 5,
                },
                SuperIoChip {
                    family: ChipFamily::It87,
                    name: "it87952".into(),
                    chip_id: 0x7591,
                    pwm_count: 5,
                    fan_count: 5,
                },
            ],
            IoKind::Helper => match lpc_client::probe() {
                Ok(chips) => chips
                    .into_iter()
                    .map(|c| SuperIoChip {
                        family: ChipFamily::It87,
                        name: c.name,
                        chip_id: c.chip_id,
                        pwm_count: c.pwm_count,
                        fan_count: c.fan_count,
                    })
                    .collect(),
                Err(e) => {
                    eprintln!("superio probe: {e}");
                    Vec::new()
                }
            },
            IoKind::Unavailable => Vec::new(),
        }
    }

    fn as_hwmon(&self, chips: &[SuperIoChip]) -> Vec<HwmonChip> {
        if matches!(lpc::kind(), IoKind::Mock) {
            let root = lpc::ensure_mock_fixture();
            return hwmon::scan(&root);
        }
        if matches!(lpc::kind(), IoKind::Helper) {
            let root = lpc::live_root();
            let _ = std::fs::create_dir_all(&root);
            if let Ok(infos) = lpc_client::probe() {
                let _ = lpc_client::materialize_from_probe(&root, &infos);
            }
            if let Ok(sensors) = lpc_client::sensors() {
                let _ = lpc_client::write_shadow(&root, &sensors);
            }
            let scanned = hwmon::scan(&root);
            if !scanned.is_empty() {
                return scanned;
            }
            // Fallback stub cards if pipe sensors failed but probe had chips.
            return chips
                .iter()
                .enumerate()
                .map(|(i, c)| stub_chip(c, &format!("hwmon{i}")))
                .collect();
        }
        chips
            .iter()
            .enumerate()
            .map(|(_i, c)| stub_chip(c, &format!("windows-superio/{:04x}", c.chip_id)))
            .collect()
    }
}

fn stub_chip(c: &SuperIoChip, path: &str) -> HwmonChip {
    let mut pwms = Vec::new();
    for i in 1..=c.pwm_count {
        pwms.push(PwmNode {
            name: format!("pwm{i}"),
            value: "0".into(),
            enable_exists: true,
            writable: true,
        });
    }
    let mut fans = Vec::new();
    for i in 1..=c.fan_count {
        fans.push(LabeledValue {
            label: format!("fan{i}"),
            value: "0".into(),
        });
    }
    HwmonChip {
        name: c.name.clone(),
        path: path.into(),
        temps: vec![LabeledValue {
            label: "CPU".into(),
            value: "0".into(),
        }],
        fans,
        pwms,
    }
}

impl CoolingBackend for WindowsSuperIoBackend {
    fn name(&self) -> &'static str {
        "windows-superio"
    }

    fn snapshot(&self) -> CoolingSnapshot {
        let _ = lpc::ensure_helper();
        let probed = self.probe();
        let mut gaps = Vec::new();
        match lpc::kind() {
            IoKind::Unavailable => {
                let pawn = if lpc::pawnio_installed() {
                    "PawnIO is installed"
                } else {
                    "PawnIO is NOT installed (winget install -e --id namazso.PawnIO)"
                };
                let daemon = if lpc::chromaflowd_installed() {
                    "chromaflowd is installed"
                } else {
                    "chromaflowd is NOT installed (elevated scripts\\install-windows-runtime.ps1)"
                };
                gaps.push(format!(
                    "no_os_control_channel: Motherboard fans blocked — chromaflow-lpc (SuperIO via PawnIO) is not installed or not running. {pawn}; {daemon}. Run elevated scripts\\install-windows-runtime.ps1."
                ));
            }
            IoKind::Mock => {
                gaps.push(
                    "lpc_lab_fixture: Cooling PWM is a FILE FIXTURE — Auto-calibrate will not spin chassis fans. Delete %LOCALAPPDATA%\\chromaflow\\run\\lpc-mock\\.lab or run scripts\\install-windows-runtime.ps1 (admin).".into(),
                );
            }
            IoKind::Helper if !lpc::bridge_live() => {
                gaps.push(
                    "lpc_pipe_down: chromaflow-lpc.exe is installed but the named pipe is not answering. Start task ChromaFlowLPC or run chromaflow-lpc --serve elevated.".into(),
                );
            }
            IoKind::Helper if probed.is_empty() => {
                gaps.push(
                    "superio_probe_empty: LPC bridge up but SuperIO at 0x2E/0x4E reads 0xFF (no ITE). On Gigabyte X570S quit EasyTune / GCC / RGB Fusion, then Support → Apply and rescan. If still empty, BIOS may have LPC config locked — PawnIO IsaBridgeEC also returns 0xFFFF."
                        .into(),
                );
            }
            _ => {}
        }
        CoolingSnapshot {
            chips: self.as_hwmon(&probed),
            source: "windows-superio",
            gaps,
        }
    }

    fn pwm_apply(&self, chip: &str, pwm: &str, duty: u8, allow_zero: bool) -> Result<(), String> {
        if duty == 0 && !allow_zero {
            return Err("0% needs an explicit confirm".into());
        }
        let root = match lpc::kind() {
            IoKind::Mock => lpc::ensure_mock_fixture(),
            IoKind::Helper => {
                let r = lpc::live_root();
                let _ = std::fs::create_dir_all(r.join(chip));
                r
            }
            IoKind::Unavailable => return Err("LPC helper unavailable".into()),
        };
        let path = root.join(chip).join(pwm);
        // apply_duty syncs Helper → pipe; enable is file-only.
        crate::pwm_apply::apply_enable(
            &path.parent().unwrap().join(format!("{pwm}_enable")),
            crate::pwm_policy::ENABLE_MANUAL,
        )?;
        crate::pwm_apply::apply_duty(&path, duty, allow_zero)
    }

    fn release(&self, chip: &str, pwm: &str) -> Result<(), String> {
        let root = match lpc::kind() {
            IoKind::Mock => lpc::ensure_mock_fixture(),
            IoKind::Helper => lpc::live_root(),
            IoKind::Unavailable => return Ok(()),
        };
        // restore_dir syncs Helper → pipe release.
        crate::pwm_apply::restore_dir(&root, chip, pwm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_snapshot_lists_pwm() {
        std::env::set_var("CHROMAFLOW_LPC_MOCK", "1");
        let snap = WindowsSuperIoBackend.snapshot();
        assert_eq!(snap.source, "windows-superio");
        assert_eq!(snap.chips.len(), 2);
        let pwm_n: usize = snap.chips.iter().map(|c| c.pwms.len()).sum();
        assert_eq!(pwm_n, 10);
        assert!(snap.chips.iter().any(|c| c.name.contains("it87952")));
        std::env::remove_var("CHROMAFLOW_LPC_MOCK");
    }
}
