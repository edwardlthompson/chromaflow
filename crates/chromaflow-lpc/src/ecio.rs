//! Gigabyte IT879x ECIO firmware-fan latch (AMD dual-ITE boards).
//!
//! Public ports/commands used by open tools on IT879x secondary EC:
//! register `0x3F4`, data `0x3F0`, read cmd `0xB0`, write cmd `0xB1`,
//! fan-control area `0x900`, enable at `+0x47`, version at `+0x00` (expect 1).
//! Original Rust — not pasted from LibreHardwareMonitor.

#![cfg(windows)]

use crate::pawnio::PawnIo;
use std::time::{Duration, Instant};

const ECIO_REG: u16 = 0x3f4;
const ECIO_VAL: u16 = 0x3f0;
const CMD_READ: u8 = 0xb0;
const CMD_WRITE: u8 = 0xb1;
const AREA: u16 = 0x900;
const REG_VERSION: u16 = AREA;
const REG_ENABLE: u16 = AREA + 0x47;
const WAIT: Duration = Duration::from_millis(1000);

fn wait_ibe(pio: &PawnIo) -> Result<(), String> {
    let start = Instant::now();
    while pio.pio_inb(ECIO_REG)? & 0x02 != 0 {
        if start.elapsed() > WAIT {
            return Err("ECIO IBE timeout".into());
        }
    }
    Ok(())
}

fn wait_obf(pio: &PawnIo) -> Result<(), String> {
    let start = Instant::now();
    while pio.pio_inb(ECIO_REG)? & 0x01 == 0 {
        if start.elapsed() > WAIT {
            return Err("ECIO OBF timeout".into());
        }
    }
    Ok(())
}

fn init(pio: &PawnIo, cmd: u8, offset: u16) -> Result<(), String> {
    wait_ibe(pio)?;
    pio.pio_outb(ECIO_REG, cmd)?;
    wait_ibe(pio)?;
    pio.pio_outb(ECIO_VAL, (offset >> 8) as u8)?;
    wait_ibe(pio)?;
    pio.pio_outb(ECIO_VAL, (offset & 0xff) as u8)?;
    Ok(())
}

fn read_u8(pio: &PawnIo, offset: u16) -> Result<u8, String> {
    init(pio, CMD_READ, offset)?;
    wait_obf(pio)?;
    pio.pio_inb(ECIO_VAL)
}

fn write_u8(pio: &PawnIo, offset: u16, value: u8) -> Result<(), String> {
    init(pio, CMD_WRITE, offset)?;
    wait_ibe(pio)?;
    pio.pio_outb(ECIO_VAL, value)
}

/// ITE enter on secondary SuperIO (0x4E) so FindBars can whitelist ECIO ports.
fn prepare_secondary_bars(pio: &PawnIo) -> Result<(), String> {
    pio.select_slot(1)?;
    // 0x4E ITE enter — leave in config (do not exit secondary).
    for b in [0x87u8, 0x01, 0x55, 0xaa] {
        pio.pio_outb(0x4e, b)?;
    }
    pio.find_bars()?;
    Ok(())
}

/// If ECIO version is 1, clear Gigabyte firmware fan enable (write 0 at 0x947).
/// Call with an LpcIO PawnIo session. Acquires the ISA bus mutex.
pub fn try_clear_firmware_fans(pio: &PawnIo) -> String {
    let _guard = match crate::isa_mutex::IsaBusGuard::acquire(Duration::from_secs(5)) {
        Ok(g) => g,
        Err(e) => return format!("ECIO ISA mutex: {e}"),
    };
    if let Err(e) = prepare_secondary_bars(pio) {
        return format!("ECIO prepare bars: {e}");
    }
    match read_u8(pio, REG_VERSION) {
        Ok(1) => {}
        Ok(v) => {
            return format!("ECIO version 0x{v:02x} (want 1) — skip");
        }
        Err(e) => return format!("ECIO version read failed: {e}"),
    }
    match read_u8(pio, REG_ENABLE) {
        Ok(0) => "ECIO Gigabyte FW latch already clear".into(),
        Ok(0xff) => "ECIO enable reads 0xFF — still locked".into(),
        Ok(v) => match write_u8(pio, REG_ENABLE, 0) {
            Ok(()) => {
                std::thread::sleep(Duration::from_millis(500));
                format!("ECIO cleared Gigabyte FW latch (was 0x{v:02x})")
            }
            Err(e) => format!("ECIO write enable failed: {e}"),
        },
        Err(e) => format!("ECIO enable read failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn public_offsets_match_documented_map() {
        assert_eq!(super::REG_ENABLE, 0x947);
        assert_eq!(super::REG_VERSION, 0x900);
        assert_eq!(super::ECIO_REG, 0x3f4);
        assert_eq!(super::ECIO_VAL, 0x3f0);
    }
}
