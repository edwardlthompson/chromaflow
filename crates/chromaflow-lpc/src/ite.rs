//! ITE SuperIO environment controller (public chip docs / it87-compatible offsets).
//! No LibreHardwareMonitor or Fan Control source.

#![cfg(windows)]

use crate::pawnio::PawnIo;
use crate::proto::{ChipInfo, Labeled, PwmReading, SensorChip};

const REG_LDN: u8 = 0x07;
const REG_CHIP_ID: u8 = 0x20;
const REG_ACTIVATE: u8 = 0x30;
const REG_BASE_H: u8 = 0x60;
const REG_BASE_L: u8 = 0x61;
const LDN_EC: u8 = 0x04;

/// IT87-family EC register offsets (software PWM mode).
const REG_FAN_MAIN_CTRL: u16 = 0x13;
const REG_FAN_CTL: u16 = 0x14;
const PWM_REGS: [u16; 5] = [0x15, 0x16, 0x17, 0x1e, 0x1f];
const FAN_LO: [u16; 5] = [0x0d, 0x0e, 0x0f, 0x80, 0x82];
const FAN_HI: [u16; 5] = [0x18, 0x19, 0x1a, 0x81, 0x83];
const TEMP_REGS: [u16; 3] = [0x29, 0x2a, 0x2b];

#[derive(Debug, Clone)]
pub struct LiveChip {
    pub info: ChipInfo,
}

fn chip_name(id: u16) -> Option<&'static str> {
    match id {
        0x8689 => Some("it8689"),
        0x8688 => Some("it8688"),
        0x8686 => Some("it8686"),
        0x8728 => Some("it8728"),
        0x8733 => Some("it8792"),
        0x8790 => Some("it8790"),
        // Gigabyte / IsaBridgeEC IT87952E id; also legacy aliases.
        0x8695 => Some("it87952"),
        0x7591 => Some("it87952"),
        0x7595 => Some("it87952"),
        0x8705 => Some("it8705"),
        0x8712 => Some("it8712"),
        0x8716 => Some("it8716"),
        0x8718 => Some("it8718"),
        0x8721 => Some("it8721"),
        0x8726 => Some("it8726"),
        0x8771 => Some("it8771"),
        0x8772 => Some("it8772"),
        _ => None,
    }
}

fn labels_for(name: &str) -> (&'static [&'static str], &'static [&'static str]) {
    if name.contains("87952") || name.contains("8792") {
        (
            &["VRM", "NVMe", "System"],
            &["FAN4", "FAN5_PUMP", "FAN6_PUMP", "FAN7_PUMP", "FAN8"],
        )
    } else if name.contains("8689") || name.contains("8688") || name.contains("8686") {
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

fn slot_index_port(slot: u8) -> u16 {
    if slot == 0 {
        0x2e
    } else {
        0x4e
    }
}

fn enter_config(pio: &PawnIo, slot: u8) -> Result<(), String> {
    // ITE MB PnP enter (public flashrom / chip-doc sequence).
    // Slot 0 (0x2E): final magic 0x55; slot 1 (0x4E): final magic 0xAA.
    let idx = slot_index_port(slot);
    let last = if slot == 0 { 0x55u8 } else { 0xaau8 };
    pio.pio_outb(idx, 0x87)?;
    pio.pio_outb(idx, 0x01)?;
    pio.pio_outb(idx, 0x55)?;
    pio.pio_outb(idx, last)?;
    Ok(())
}

fn exit_config(pio: &PawnIo, slot: u8) -> Result<(), String> {
    // Secondary SuperIO (0x4E) must stay in config mode — exiting breaks IT879x.
    if slot != 0 {
        return Ok(());
    }
    let idx = slot_index_port(slot);
    pio.pio_outb(idx, 0x02)?;
    pio.pio_outb(idx.wrapping_add(1), 0x02)?;
    Ok(())
}

fn ec_base_plausible(base: u16) -> bool {
    // Same checks used by open ITE detectors: not zero/FFFF, not below 0x100, aligned.
    base >= 0x100 && base != 0xffff && (base & 0x0007) == 0
}

fn read_ec_base(pio: &PawnIo) -> Result<u16, String> {
    pio.sio_outb(REG_LDN, LDN_EC)?;
    let hi = pio.sio_inb(REG_BASE_H)?;
    let lo = pio.sio_inb(REG_BASE_L)?;
    let base = (u16::from(hi) << 8) | u16::from(lo);
    if !ec_base_plausible(base) {
        return Err(format!("EC base invalid 0x{base:04x}"));
    }
    Ok(base)
}

/// Gigabyte / IT87 EC IO bases commonly exposed after FindBars (public hwmon ranges).
const EC_BASE_CANDIDATES: &[u16] = &[
    0x0a20, 0x0a30, 0x0a40, 0x0a45, 0x0a50, 0x0a60, 0x0a70, 0x0a80, 0x0a00, 0x0220, 0x0290,
    0x0ae0, 0x0af0, 0x0aa0, 0x0ab0, 0x0ac0, 0x0ad0, 0x0b00, 0x0b10, 0x0900, 0x0910,
];

fn ec_candidate_alive(pio: &PawnIo, base: u16) -> bool {
    if !ec_base_plausible(base) {
        return false;
    }
    // Chip version / config at 0x00–0x01 often non-FF when the EC window is real.
    let Ok(v0) = pio.pio_inb(base) else {
        return false;
    };
    let Ok(v1) = pio.pio_inb(base.wrapping_add(1)) else {
        return false;
    };
    if v0 == 0xff && v1 == 0xff {
        return false;
    }
    // Fan1 count registers — all-FF usually means no decode.
    let Ok(lo) = pio.pio_inb(base.wrapping_add(FAN_LO[0])) else {
        return false;
    };
    let Ok(hi) = pio.pio_inb(base.wrapping_add(FAN_HI[0])) else {
        return false;
    };
    !(lo == 0xff && hi == 0xff && v0 == 0xff)
}

/// Gigabyte often leaves LDN EC base as a chip-ID echo (e.g. 0x8686). FindBars then
/// never whitelists the real EC window. Program each candidate into LDN 4, FindBars,
/// and test — same idea as public ITE detectors that rewrite 0x60/0x61 when garbage.
fn program_ec_base(pio: &PawnIo, base: u16) -> Result<u16, String> {
    pio.sio_outb(REG_LDN, LDN_EC)?;
    pio.sio_outb(REG_BASE_H, (base >> 8) as u8)?;
    pio.sio_outb(REG_BASE_L, (base & 0xff) as u8)?;
    // Ensure logical device active (public ITE LDN activate register).
    let _ = pio.sio_outb(REG_ACTIVATE, 0x01);
    let hi = pio.sio_inb(REG_BASE_H)?;
    let lo = pio.sio_inb(REG_BASE_L)?;
    let readback = (u16::from(hi) << 8) | u16::from(lo);
    pio.find_bars()?;
    Ok(readback)
}

fn find_ec_base_via_bars(pio: &PawnIo) -> Result<u16, String> {
    match pio.find_bars() {
        Ok(()) => {}
        Err(e) => return Err(format!("find_bars: {e}")),
    }
    for &base in EC_BASE_CANDIDATES {
        if ec_candidate_alive(pio, base) {
            eprintln!("chromaflow-lpc: EC base candidate 0x{base:04x} looks live");
            return Ok(base);
        }
    }
    // LDN base was garbage — rewrite + FindBars so PawnIO whitelists the port.
    let mut denied = 0u32;
    let mut dead = 0u32;
    let mut stuck_rb = 0u32;
    for &base in EC_BASE_CANDIDATES {
        let readback = match program_ec_base(pio, base) {
            Ok(rb) => rb,
            Err(e) => {
                eprintln!("chromaflow-lpc: program EC 0x{base:04x}: {e}");
                continue;
            }
        };
        if readback != base {
            stuck_rb += 1;
            if stuck_rb <= 3 {
                eprintln!(
                    "chromaflow-lpc: LDN EC write 0x{base:04x} readback 0x{readback:04x} (locked?)"
                );
            }
        }
        if ec_candidate_alive(pio, base) {
            eprintln!("chromaflow-lpc: programmed EC base 0x{base:04x} looks live");
            return Ok(base);
        }
        // Distinguish ACCESS_DENIED (still not whitelisted) from open-but-dead.
        match pio.pio_inb(base) {
            Err(_) => denied += 1,
            Ok(_) => dead += 1,
        }
    }
    // Dump which ports FindBars actually whitelisted (helps Gigabyte ghost-ID boards).
    let mut allowed: Vec<u16> = Vec::new();
    let mut p = 0x0100u16;
    while p < 0xf000 {
        if pio.pio_inb(p).is_ok() {
            allowed.push(p);
            if allowed.len() >= 48 {
                break;
            }
        }
        p = p.saturating_add(1);
        if p >= 0x9000 && allowed.is_empty() {
            // Skip sparse high IO if nothing showed up in low ranges.
            break;
        }
    }
    eprintln!(
        "chromaflow-lpc: FindBars-allowed sample ({}) {:?}",
        allowed.len(),
        &allowed[..allowed.len().min(24)]
    );
    for &base in &allowed {
        if ec_base_plausible(base) && ec_candidate_alive(pio, base) {
            eprintln!("chromaflow-lpc: allowed-port EC base 0x{base:04x} looks live");
            return Ok(base);
        }
    }
    Err(format!(
        "no live EC base after LDN program (denied={denied} dead={dead} stuck_rb={stuck_rb} allowed={})",
        allowed.len()
    ))
}

/// Last-resort: ITE-enter (may flip chip ID to 0xFFFF on Gigabyte), program EC base,
/// FindBars, then exit — unlocks LDN writes when BIOS left config registers read-only.
fn recover_ec_base_with_enter(pio: &PawnIo, slot: u8) -> Result<u16, String> {
    enter_config(pio, slot)?;
    let id_after = read_chip_id(pio).unwrap_or(0xffff);
    eprintln!("chromaflow-lpc: enter recover slot {slot} id=0x{id_after:04x}");
    let mut last_err = String::from("no candidate");
    for &base in EC_BASE_CANDIDATES {
        match program_ec_base(pio, base) {
            Ok(rb) if rb == base && ec_candidate_alive(pio, base) => {
                let _ = exit_config(pio, slot);
                eprintln!("chromaflow-lpc: enter-recover EC base 0x{base:04x}");
                return Ok(base);
            }
            Ok(rb) => {
                last_err = format!("0x{base:04x} rb=0x{rb:04x} not live");
            }
            Err(e) => last_err = e,
        }
    }
    let _ = exit_config(pio, slot);
    Err(last_err)
}

fn ec_in(pio: &PawnIo, base: u16, off: u16) -> Result<u8, String> {
    pio.pio_inb(base.wrapping_add(off))
}

fn ec_out(pio: &PawnIo, base: u16, off: u16, val: u8) -> Result<(), String> {
    pio.pio_outb(base.wrapping_add(off), val)
}

fn fan_rpm(pio: &PawnIo, base: u16, idx: usize) -> Result<u32, String> {
    let lo = ec_in(pio, base, FAN_LO[idx])?;
    let hi = ec_in(pio, base, FAN_HI[idx])?;
    let raw = (u32::from(hi) << 8) | u32::from(lo);
    if raw == 0 || raw == 0xffff {
        return Ok(0);
    }
    // IT87 16-bit mode: RPM = 1.35e6 / count (public it87 formula).
    Ok((1_350_000u32 / raw).min(30_000))
}

fn temp_mc(pio: &PawnIo, base: u16, idx: usize) -> Result<i32, String> {
    let t = ec_in(pio, base, TEMP_REGS[idx])? as i8;
    Ok(i32::from(t) * 1000)
}

/// Gigabyte dual-ITE: secondary (0x4E) must be in config before primary (0x2E) or
/// LPC returns ghost IDs / 0xFFFF (public it87 Gigabyte quirk). Never exit slot 1.
fn ensure_secondary_config(pio: &PawnIo) {
    if pio.select_slot(1).is_err() {
        return;
    }
    let id = read_chip_id(pio).unwrap_or(0xffff);
    if id != 0 && id != 0xffff && chip_name(id).is_some() {
        eprintln!("chromaflow-lpc: secondary already open id=0x{id:04x}");
        return;
    }
    match enter_config(pio, 1) {
        Ok(()) => {
            let after = read_chip_id(pio).unwrap_or(0xffff);
            eprintln!("chromaflow-lpc: secondary enter id=0x{after:04x}");
        }
        Err(e) => eprintln!("chromaflow-lpc: secondary enter: {e}"),
    }
}

/// True when chip-ID bytes look mirrored and EC base echoes the same — not real config.
fn is_ghost_sio_id(pio: &PawnIo, id: u16) -> bool {
    if id == 0 || id == 0xffff {
        return false;
    }
    let equal = (id >> 8) == (id & 0xff);
    if !equal {
        return false;
    }
    match read_ec_base(pio) {
        Err(_) => true,
        Ok(b) => b == id || !ec_base_plausible(b),
    }
}

/// Probe both SuperIO slots for known ITE chips.
pub fn probe(pio: &PawnIo) -> Result<Vec<LiveChip>, String> {
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    ensure_secondary_config(pio);
    let mut out = Vec::new();
    let mut hwmon_i = 0u8;
    // Secondary first, then primary (Gigabyte LPC ordering).
    for slot in [1u8, 0u8] {
        if let Err(e) = pio.select_slot(slot) {
            eprintln!("chromaflow-lpc: select_slot {slot}: {e}");
            continue;
        }
        let mut id = read_chip_id(pio).unwrap_or(0xffff);
        let mut we_entered = false;
        // Slot 1: enter only if closed; stay in config (exit_config no-ops slot 1).
        // Slot 0: enter when empty/FFFF/ghost — never trust mirrored 0x8686 as live config.
        let need_enter = id == 0
            || id == 0xffff
            || (slot == 0 && is_ghost_sio_id(pio, id))
            || (slot == 1 && chip_name(id).is_none());
        if need_enter {
            if let Err(e) = enter_config(pio, slot) {
                eprintln!("chromaflow-lpc: enter_config slot {slot}: {e}");
                continue;
            }
            we_entered = true;
            id = match read_chip_id(pio) {
                Ok(v) => v,
                Err(e) => {
                    let _ = exit_config(pio, slot);
                    eprintln!("chromaflow-lpc: chip id slot {slot}: {e}");
                    continue;
                }
            };
            eprintln!("chromaflow-lpc: slot {slot} after enter id=0x{id:04x}");
        } else {
            eprintln!("chromaflow-lpc: slot {slot} already in config, id=0x{id:04x}");
        }
        if chip_name(id).is_none() {
            if we_entered {
                let _ = exit_config(pio, slot);
            }
            eprintln!("chromaflow-lpc: unknown chip 0x{id:04x} slot {slot}");
            continue;
        }
        let base = match read_ec_base(pio) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("chromaflow-lpc: EC base slot {slot}: {e}; scanning FindBars candidates");
                match find_ec_base_via_bars(pio) {
                    Ok(b) => b,
                    Err(ee) => {
                        // Enter-recover on primary while secondary is dead yields 0xFFFF and
                        // burns the Gigabyte LPC bus for the rest of the boot — skip it.
                        if slot == 0 {
                            eprintln!(
                                "chromaflow-lpc: EC base slot {slot}: {ee}; skip enter-recover (secondary-first)"
                            );
                            if we_entered {
                                let _ = exit_config(pio, slot);
                            }
                            continue;
                        }
                        eprintln!("chromaflow-lpc: EC base slot {slot}: {ee}; trying enter-recover");
                        match recover_ec_base_with_enter(pio, slot) {
                            Ok(b) => {
                                we_entered = true;
                                b
                            }
                            Err(eee) => {
                                if we_entered {
                                    let _ = exit_config(pio, slot);
                                }
                                eprintln!("chromaflow-lpc: EC base slot {slot}: {eee}");
                                continue;
                            }
                        }
                    }
                }
            }
        };
        // Prefer silkscreen names used on X570S when ID echoed as 0x8686.
        if id == 0x8686 {
            id = 0x8689;
        }
        let name = chip_name(id).unwrap_or("it8689");
        if let Err(e) = pio.find_bars() {
            eprintln!("chromaflow-lpc: find_bars slot {slot}: {e} (continuing with EC 0x{base:04x})");
        }
        // Only exit primary if we entered it — never exit secondary.
        if we_entered {
            let _ = exit_config(pio, slot);
        }
        let dir = format!("hwmon{hwmon_i}");
        hwmon_i += 1;
        out.push(LiveChip {
            info: ChipInfo {
                name: name.into(),
                chip_id: id,
                slot,
                pwm_count: 5,
                fan_count: 5,
                hwmon_dir: dir,
                ec_base: base,
            },
        });
    }
    if out.is_empty() {
        return Err("no ITE SuperIO chips found on slots 0/1".into());
    }
    Ok(out)
}

fn read_chip_id(pio: &PawnIo) -> Result<u16, String> {
    // Prefer word read; fall back to two byte reads (same public 0x20/0x21 map).
    if let Ok(id) = pio.sio_inw(REG_CHIP_ID) {
        if id != 0 && id != 0xffff {
            return Ok(id);
        }
    }
    let hi = pio.sio_inb(REG_CHIP_ID)?;
    let lo = pio.sio_inb(REG_CHIP_ID + 1)?;
    Ok((u16::from(hi) << 8) | u16::from(lo))
}

pub fn read_sensors(pio: &PawnIo, chips: &[LiveChip]) -> Result<Vec<SensorChip>, String> {
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    let mut sensors = Vec::new();
    for chip in chips {
        pio.select_slot(chip.info.slot)?;
        // EC data ports are whitelisted via find_bars while in config once at probe;
        // re-enter + find_bars so BARs stay allowed across reconnects.
        let _ = enter_config(pio, chip.info.slot);
        let _ = pio.find_bars();
        let _ = exit_config(pio, chip.info.slot);
        let base = chip.info.ec_base;
        let (temps_l, fans_l) = labels_for(&chip.info.name);
        let mut temps = Vec::new();
        for (i, label) in temps_l.iter().enumerate().take(3) {
            let mc = temp_mc(pio, base, i).unwrap_or(0);
            temps.push(Labeled {
                label: (*label).into(),
                value: mc.to_string(),
            });
        }
        let mut fans = Vec::new();
        for (i, label) in fans_l.iter().enumerate().take(5) {
            let rpm = fan_rpm(pio, base, i).unwrap_or(0);
            fans.push(Labeled {
                label: (*label).into(),
                value: rpm.to_string(),
            });
        }
        let mut pwms = Vec::new();
        for i in 0..5usize {
            let duty = ec_in(pio, base, PWM_REGS[i]).unwrap_or(0);
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

fn chip_by_dir<'a>(chips: &'a [LiveChip], dir: &str) -> Result<&'a LiveChip, String> {
    chips
        .iter()
        .find(|c| c.info.hwmon_dir == dir)
        .ok_or_else(|| format!("unknown dir {dir}"))
}

/// Reject silent 0% unless the caller confirmed (same contract as Linux pwm_apply).
pub fn check_duty(duty: u8, allow_zero: bool) -> Result<(), String> {
    if duty == 0 && !allow_zero {
        return Err("0% needs an explicit confirm".into());
    }
    Ok(())
}

/// Force software PWM mode and write duty (0–255). Never writes 0 unless caller allows.
pub fn pwm_apply(
    pio: &PawnIo,
    chips: &[LiveChip],
    dir: &str,
    pwm: &str,
    duty: u8,
    allow_zero: bool,
) -> Result<(), String> {
    check_duty(duty, allow_zero)?;
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    let chip = chip_by_dir(chips, dir)?;
    let idx = pwm_index(pwm)?;
    ensure_bars(pio, chip.info.slot)?;
    let base = chip.info.ec_base;
    // Enable smartguardian / software control bits (public it87 FAN_MAIN_CTRL / FAN_CTL).
    let mut main = ec_in(pio, base, REG_FAN_MAIN_CTRL).unwrap_or(0);
    main |= 0x07; // enable fan monitoring bits commonly used on IT87
    ec_out(pio, base, REG_FAN_MAIN_CTRL, main)?;
    let mut ctl = ec_in(pio, base, REG_FAN_CTL).unwrap_or(0);
    ctl |= 0x80; // software/manual style control (IT87 family)
    ec_out(pio, base, REG_FAN_CTL, ctl)?;
    ec_out(pio, base, PWM_REGS[idx], duty)?;
    Ok(())
}

fn ensure_bars(pio: &PawnIo, slot: u8) -> Result<(), String> {
    pio.select_slot(slot)?;
    // Slot 1 may already be in config; only enter when chip ID looks unentered.
    let pre = read_chip_id(pio).unwrap_or(0xffff);
    if slot == 0 || pre == 0 || pre == 0xffff {
        enter_config(pio, slot)?;
    }
    pio.find_bars()?;
    exit_config(pio, slot)?;
    Ok(())
}

/// Restore automatic / firmware-ish control for one channel (best-effort).
pub fn release(pio: &PawnIo, chips: &[LiveChip], dir: &str, pwm: &str) -> Result<(), String> {
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    let chip = chip_by_dir(chips, dir)?;
    let idx = pwm_index(pwm)?;
    ensure_bars(pio, chip.info.slot)?;
    let base = chip.info.ec_base;
    // Clear software-override vibe: leave duty, clear manual-ish FAN_CTL high bit when releasing last.
    let _ = idx;
    let mut ctl = ec_in(pio, base, REG_FAN_CTL).unwrap_or(0);
    ctl &= !0x80;
    ec_out(pio, base, REG_FAN_CTL, ctl)?;
    Ok(())
}

pub fn release_all(pio: &PawnIo, chips: &[LiveChip]) {
    for c in chips {
        for i in 1..=5u8 {
            let _ = release(pio, chips, &c.info.hwmon_dir, &format!("pwm{i}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_duty, pwm_index};

    #[test]
    fn pwm_index_ok() {
        assert_eq!(pwm_index("pwm1").unwrap(), 0);
        assert_eq!(pwm_index("pwm5").unwrap(), 4);
        assert!(pwm_index("pwm0").is_err());
        assert!(pwm_index("fan1").is_err());
    }

    #[test]
    fn duty_never_silent_zero() {
        assert!(check_duty(0, false).is_err());
        assert!(check_duty(0, true).is_ok());
        assert!(check_duty(128, false).is_ok());
        assert!(check_duty(255, false).is_ok());
    }
}
