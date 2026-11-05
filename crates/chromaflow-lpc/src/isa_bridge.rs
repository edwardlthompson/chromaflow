//! Gigabyte-style SuperIO via PawnIO IsaBridgeEC (MMIO). Used when classic LpcIO
//! chip-ID reads return 0xFFFF on AMD boards that need LPC MMIO decode.

#![cfg(windows)]

use crate::pawnio::PawnIo;
use crate::proto::{ChipInfo, Labeled, PwmReading, SensorChip};

/// AccessType for ioctl_access_superio_mmio.
const ACCESS_READ: u64 = 0;
const ACCESS_WRITE: u64 = 1;

/// IT87-family EC register offsets (same public map as classic IO space).
const REG_FAN_MAIN_CTRL: u64 = 0x13;
const REG_FAN_CTL: u64 = 0x14;
const PWM_REGS: [u64; 5] = [0x15, 0x16, 0x17, 0x1e, 0x1f];
const FAN_LO: [u64; 5] = [0x0d, 0x0e, 0x0f, 0x80, 0x82];
const FAN_HI: [u64; 5] = [0x18, 0x19, 0x1a, 0x81, 0x83];
const TEMP_REGS: [u64; 3] = [0x29, 0x2a, 0x2b];

#[derive(Debug, Clone)]
pub struct MmioChip {
    pub info: ChipInfo,
}

fn chip_name(id: u16) -> Option<&'static str> {
    match id {
        0x8689 => Some("it8689"),
        0x8688 => Some("it8688"),
        0x8686 => Some("it8686"),
        0x8696 => Some("it8696"),
        0x8698 => Some("it8698"),
        0x8790 => Some("it8790"),
        0x8733 => Some("it8792"),
        // IsaBridgeEC / Gigabyte IT87952E device id
        0x8695 => Some("it87952"),
        0x7591 | 0x7595 => Some("it87952"),
        0x8728 => Some("it8728"),
        _ => None,
    }
}

fn labels_for(name: &str) -> (&'static [&'static str], &'static [&'static str]) {
    if name.contains("87952") || name.contains("8792") || name.contains("8790") {
        (
            &["VRM", "NVMe", "System"],
            &["FAN4", "FAN5_PUMP", "FAN6_PUMP", "FAN7_PUMP", "FAN8"],
        )
    } else if name.contains("8689") || name.contains("8688") {
        (
            &["CPU", "DIMM", "System"],
            &["CPU_FAN", "SYS_FAN1", "SYS_FAN2", "SYS_FAN3", "CPU_OPT"],
        )
    } else {
        (
            &["temp1", "temp2", "temp3"],
            &["fan1", "fan2", "fan3", "fan4", "fan5"],
        )
    }
}

fn parse_find(out: &[u64]) -> Vec<(u8, u64, u64, u16)> {
    let mut rows = Vec::new();
    for slot in 0u8..=1 {
        let i = slot as usize * 3;
        if out.len() < i + 3 {
            break;
        }
        rows.push((slot, out[i], out[i + 1], out[i + 2] as u16));
    }
    rows
}

fn chips_from_find(out: &[u64]) -> Vec<MmioChip> {
    let mut chips = Vec::new();
    let mut hwmon_i = 0u8;
    for (slot, base, size, id) in parse_find(out) {
        eprintln!(
            "chromaflow-lpc: IsaBridgeEC slot {slot}: base=0x{base:x} size=0x{size:x} id=0x{id:04x}"
        );
        let Some(name) = chip_name(id) else {
            continue;
        };
        if base == 0 || size == 0 {
            eprintln!("chromaflow-lpc: IsaBridgeEC slot {slot}: chip {name} without MMIO base");
        }
        let dir = format!("hwmon{hwmon_i}");
        hwmon_i += 1;
        chips.push(MmioChip {
            info: ChipInfo {
                name: name.into(),
                chip_id: id,
                slot,
                pwm_count: 5,
                fan_count: 5,
                hwmon_dir: dir,
                // Store MMIO base in ec_base for access_superio_mmio offset math (low 16 unused).
                ec_base: (base & 0xffff) as u16,
            },
        });
    }
    chips
}

/// Discover chips via ioctl_find_superio_mmio (unrestricted SuperIO enter inside module).
/// Retries after enabling MMIO decode states — Gigabyte AMD often needs set_state before find.
pub fn probe(pio: &PawnIo) -> Result<Vec<MmioChip>, String> {
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    let mut best = pio.execute("ioctl_find_superio_mmio", &[], 6)?;
    let mut chips = chips_from_find(&best);
    if chips.is_empty() {
        for state in [1u64, 2u64, 0u64] {
            let _ = pio.execute("ioctl_iomem_mmio_set_state", &[state], 0);
            let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
            if let Ok(out) = pio.execute("ioctl_find_superio_mmio", &[], 6) {
                let next = chips_from_find(&out);
                if !next.is_empty() {
                    best = out;
                    chips = next;
                    break;
                }
                best = out;
            }
        }
    }
    if best.len() < 6 && chips.is_empty() {
        return Err(format!("find_superio_mmio short out ({})", best.len()));
    }
    // Even with unknown IDs, clearing Gigabyte FW latch can unlock classic LpcIO on retry.
    disable_gigabyte_firmware_fans(pio, &best);
    if chips.is_empty() {
        // Re-find after FW clear — chip IDs sometimes appear only then.
        if let Ok(out) = pio.execute("ioctl_find_superio_mmio", &[], 6) {
            chips = chips_from_find(&out);
        }
    }
    if chips.is_empty() {
        // Honest empty (IDs often 0xFFFF when LPC/MMIO is locked) — not a hard pipe error.
        eprintln!("chromaflow-lpc: IsaBridgeEC: no known ITE chips (ids may be 0xFFFF)");
        return Ok(Vec::new());
    }
    let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
    for state in [1u64, 2u64] {
        let _ = pio.execute("ioctl_iomem_mmio_set_state", &[state], 0);
    }
    let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
    Ok(chips)
}

/// Gigabyte custom EC fan latch in SMFI/MMIO space (area 0x900 + enable 0x47).
/// Public behavior: write 0 so IT87 PWM registers answer (Fan Control / LHM PR discussions;
/// original Rust — not pasted from LHM).
const GB_FAN_FW_OFFSET: u64 = 0x900 + 0x47;

/// Disable Gigabyte firmware fan ownership on whichever SuperIO slot has a mapped base.
fn disable_gigabyte_firmware_fans(pio: &PawnIo, find_out: &[u64]) {
    let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
    let _ = pio.execute("ioctl_iomem_mmio_set_state", &[2u64], 0); // MMIO_Enabled4E
    let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
    for (slot, base, size, id) in parse_find(find_out) {
        if base == 0 || size == 0 {
            continue;
        }
        match mmio_read_u8(pio, slot, GB_FAN_FW_OFFSET) {
            Ok(0xff) => {
                eprintln!(
                    "chromaflow-lpc: Gigabyte FW latch slot {slot} id=0x{id:04x} reads 0xFF (skip)"
                );
            }
            Ok(v) if v != 0 => {
                eprintln!(
                    "chromaflow-lpc: Gigabyte FW latch slot {slot} was 0x{v:02x} — clearing for IT87 PWM"
                );
                if let Err(e) = mmio_write_u8(pio, slot, GB_FAN_FW_OFFSET, 0) {
                    eprintln!("chromaflow-lpc: Gigabyte FW clear failed: {e}");
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                }
            }
            Ok(_) => {
                eprintln!("chromaflow-lpc: Gigabyte FW latch slot {slot} already clear");
            }
            Err(e) => {
                eprintln!("chromaflow-lpc: Gigabyte FW read slot {slot}: {e}");
            }
        }
    }
}

fn mmio_read_u8(pio: &PawnIo, slot: u8, offset: u64) -> Result<u8, String> {
    let out = pio.execute(
        "ioctl_access_superio_mmio",
        &[u64::from(slot), offset, 1, ACCESS_READ, 0],
        1,
    )?;
    Ok(out.first().copied().unwrap_or(0) as u8)
}

fn mmio_write_u8(pio: &PawnIo, slot: u8, offset: u64, value: u8) -> Result<(), String> {
    pio.execute(
        "ioctl_access_superio_mmio",
        &[u64::from(slot), offset, 1, ACCESS_WRITE, u64::from(value)],
        1,
    )?;
    Ok(())
}

fn fan_rpm(pio: &PawnIo, slot: u8, idx: usize) -> Result<u32, String> {
    let lo = mmio_read_u8(pio, slot, FAN_LO[idx])?;
    let hi = mmio_read_u8(pio, slot, FAN_HI[idx])?;
    let raw = (u32::from(hi) << 8) | u32::from(lo);
    if raw == 0 || raw == 0xffff {
        return Ok(0);
    }
    Ok((1_350_000u32 / raw).min(30_000))
}

pub fn read_sensors(pio: &PawnIo, chips: &[MmioChip]) -> Result<Vec<SensorChip>, String> {
    let mut sensors = Vec::new();
    for chip in chips {
        let slot = chip.info.slot;
        let (temps_l, fans_l) = labels_for(&chip.info.name);
        let mut temps = Vec::new();
        for (i, label) in temps_l.iter().enumerate().take(3) {
            let t = mmio_read_u8(pio, slot, TEMP_REGS[i]).unwrap_or(0) as i8;
            temps.push(Labeled {
                label: (*label).into(),
                value: (i32::from(t) * 1000).to_string(),
            });
        }
        let mut fans = Vec::new();
        for (i, label) in fans_l.iter().enumerate().take(5) {
            let rpm = fan_rpm(pio, slot, i).unwrap_or(0);
            fans.push(Labeled {
                label: (*label).into(),
                value: rpm.to_string(),
            });
        }
        let mut pwms = Vec::new();
        for i in 0..5usize {
            let duty = mmio_read_u8(pio, slot, PWM_REGS[i]).unwrap_or(0);
            pwms.push(PwmReading {
                name: format!("pwm{}", i + 1),
                duty,
                enable: 1,
            });
        }
        sensors.push(SensorChip {
            dir: chip.info.hwmon_dir.clone(),
            temps,
            fans,
            pwms,
        });
    }
    Ok(sensors)
}

fn pwm_index(pwm: &str) -> Result<usize, String> {
    let n = pwm
        .strip_prefix("pwm")
        .ok_or_else(|| format!("bad pwm {pwm}"))?
        .parse::<usize>()
        .map_err(|_| format!("bad pwm {pwm}"))?;
    if (1..=5).contains(&n) {
        Ok(n - 1)
    } else {
        Err(format!("pwm out of range {pwm}"))
    }
}

fn chip_by_dir<'a>(chips: &'a [MmioChip], dir: &str) -> Result<&'a MmioChip, String> {
    chips
        .iter()
        .find(|c| c.info.hwmon_dir == dir)
        .ok_or_else(|| format!("unknown dir {dir}"))
}

pub fn pwm_apply(
    pio: &PawnIo,
    chips: &[MmioChip],
    dir: &str,
    pwm: &str,
    duty: u8,
    allow_zero: bool,
) -> Result<(), String> {
    crate::ite::check_duty(duty, allow_zero)?;
    let chip = chip_by_dir(chips, dir)?;
    let idx = pwm_index(pwm)?;
    let slot = chip.info.slot;
    let mut main = mmio_read_u8(pio, slot, REG_FAN_MAIN_CTRL).unwrap_or(0);
    main |= 0x07;
    mmio_write_u8(pio, slot, REG_FAN_MAIN_CTRL, main)?;
    let mut ctl = mmio_read_u8(pio, slot, REG_FAN_CTL).unwrap_or(0);
    ctl |= 0x80;
    mmio_write_u8(pio, slot, REG_FAN_CTL, ctl)?;
    mmio_write_u8(pio, slot, PWM_REGS[idx], duty)?;
    Ok(())
}

pub fn release(pio: &PawnIo, chips: &[MmioChip], dir: &str, pwm: &str) -> Result<(), String> {
    let chip = chip_by_dir(chips, dir)?;
    let _ = pwm_index(pwm)?;
    let slot = chip.info.slot;
    let mut ctl = mmio_read_u8(pio, slot, REG_FAN_CTL).unwrap_or(0);
    ctl &= !0x80;
    mmio_write_u8(pio, slot, REG_FAN_CTL, ctl)?;
    Ok(())
}

pub fn release_all(pio: &PawnIo, chips: &[MmioChip]) {
    for c in chips {
        for i in 1..=5u8 {
            let _ = release(pio, chips, &c.info.hwmon_dir, &format!("pwm{i}"));
        }
    }
    let _ = pio.execute("ioctl_unmap_superio_mmio", &[], 0);
    let _ = pio.execute("ioctl_iomem_mmio_set_state", &[0u64], 0); // Disabled / best-effort restore
}
