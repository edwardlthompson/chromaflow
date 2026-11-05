//! One-shot elevated diagnostics for SuperIO / IsaBridgeEC (no named pipe).

#![cfg(windows)]

use crate::pawnio::PawnIo;
use std::path::Path;

fn hr_line(label: &str, result: Result<Vec<u64>, String>) {
    match result {
        Ok(v) => {
            let parts: Vec<String> = v.iter().map(|x| format!("0x{x:X}")).collect();
            eprintln!("OK  {label}: [{}]", parts.join(", "));
        }
        Err(e) => eprintln!("ERR {label}: {e}"),
    }
}

fn try_lpc(dll: &Path, blob: &[u8]) -> Result<(), String> {
    eprintln!("=== LpcIO ===");
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))
        .map_err(|e| format!("ISA mutex: {e}"))?;
    let pio = PawnIo::open(dll, blob)?;
    // Gigabyte: enter secondary first and leave it open before touching 0x2E.
    eprintln!("--- Gigabyte secondary-first enter (0x4E) ---");
    let _ = pio.execute("ioctl_select_slot", &[1u64], 0);
    for b in [0x87u8, 0x01, 0x55, 0xaa] {
        let _ = pio.execute("ioctl_pio_outb", &[0x4eu64, u64::from(b)], 0);
    }
    hr_line(
        "superio_inw(0x20) secondary after enter",
        pio.execute("ioctl_superio_inw", &[0x20], 1),
    );
    // Slot order: secondary then primary (matches ite::probe).
    for slot in [1u8, 0u8] {
        let idx = if slot == 0 { 0x2eu16 } else { 0x4eu16 };
        let last = if slot == 0 { 0x55u8 } else { 0xaau8 };
        eprintln!("--- slot {slot} (index 0x{idx:02X}) ---");
        hr_line(
            "ioctl_select_slot",
            pio.execute("ioctl_select_slot", &[u64::from(slot)], 0)
                .map(|_| vec![]),
        );
        // Before enter — Fan Control may leave chips already configured.
        hr_line(
            "pio_inb idx (pre)",
            pio.execute("ioctl_pio_inb", &[u64::from(idx)], 1),
        );
        hr_line(
            "superio_inw(0x20) pre-enter",
            pio.execute("ioctl_superio_inw", &[0x20], 1),
        );
        hr_line(
            "superio_inb(0x20) pre-enter",
            pio.execute("ioctl_superio_inb", &[0x20], 1),
        );
        hr_line(
            "superio_inb(0x21) pre-enter",
            pio.execute("ioctl_superio_inb", &[0x21], 1),
        );
        match pio.execute("ioctl_find_bars", &[], 0) {
            Ok(_) => eprintln!("OK  ioctl_find_bars (pre-enter)"),
            Err(e) => eprintln!("ERR ioctl_find_bars (pre-enter): {e}"),
        }
        // LDN 4 EC base + try common Gigabyte EC ports (need BAR whitelist).
        let _ = pio.execute("ioctl_superio_outb", &[0x07, 0x04], 0);
        hr_line(
            "LDN4 baseH (0x60)",
            pio.execute("ioctl_superio_inb", &[0x60], 1),
        );
        hr_line(
            "LDN4 baseL (0x61)",
            pio.execute("ioctl_superio_inb", &[0x61], 1),
        );
        for port in [0x0a40u64, 0x0a41, 0x0a45, 0x0a60, 0x0a61, 0x0290, 0x0295] {
            hr_line(
                &format!("pio_inb 0x{port:X}"),
                pio.execute("ioctl_pio_inb", &[port], 1),
            );
        }
        // Winbond-style enter (0x87,0x87) — LHM tries this before ITE.
        let _ = pio.execute("ioctl_pio_outb", &[u64::from(idx), 0x87], 0);
        let _ = pio.execute("ioctl_pio_outb", &[u64::from(idx), 0x87], 0);
        hr_line(
            "superio_inw(0x20) post-winbond",
            pio.execute("ioctl_superio_inw", &[0x20], 1),
        );
        // ITE enter (may return 0xFFFF on this board if Winbond already confused the bus).
        for b in [0x87u8, 0x01, 0x55, last] {
            let _ = pio.execute("ioctl_pio_outb", &[u64::from(idx), u64::from(b)], 0);
        }
        hr_line(
            "superio_inw(0x20) post-ite",
            pio.execute("ioctl_superio_inw", &[0x20], 1),
        );
        hr_line(
            "superio_inb(0x20) post-ite",
            pio.execute("ioctl_superio_inb", &[0x20], 1),
        );
        if slot == 0 {
            let _ = pio.execute("ioctl_pio_outb", &[u64::from(idx), 0x02], 0);
            let _ = pio.execute(
                "ioctl_pio_outb",
                &[u64::from(idx.wrapping_add(1)), 0x02],
                0,
            );
        }
    }
    Ok(())
}

fn try_isa(dll: &Path, blob: &[u8]) -> Result<(), String> {
    eprintln!("=== IsaBridgeEC ===");
    let _isa = crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))
        .map_err(|e| format!("ISA mutex: {e}"))?;
    let pio = PawnIo::open(dll, blob)?;
    // Find before any LpcIO enter (IsaBridge uses unrestricted PIO).
    hr_line(
        "ioctl_find_superio_mmio",
        pio.execute("ioctl_find_superio_mmio", &[], 6),
    );
    hr_line(
        "ioctl_iomem_mmio_get_org_state",
        pio.execute("ioctl_iomem_mmio_get_org_state", &[], 1),
    );
    hr_line(
        "ioctl_iomem_mmio_get_cur_state",
        pio.execute("ioctl_iomem_mmio_get_cur_state", &[], 1),
    );
    // Attempt enable even if find failed — expect NOT_SUPPORTED without bases.
    for state in [0u64, 1, 2] {
        hr_line(
            &format!("ioctl_iomem_mmio_set_state({state})"),
            pio.execute("ioctl_iomem_mmio_set_state", &[state], 0)
                .map(|_| vec![]),
        );
        hr_line(
            "ioctl_find_superio_mmio (after set)",
            pio.execute("ioctl_find_superio_mmio", &[], 6),
        );
    }
    Ok(())
}

/// Enumerate PCI devices whose class looks like ISA/LPC bridge (no port IO).
fn dump_pci_bridges() {
    eprintln!("=== PCI ISA/LPC bridges (CIM) ===");
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
Get-CimInstance Win32_PnPEntity |
  Where-Object { $_.PNPClass -eq 'System' -or $_.Name -match 'LPC|ISA|FCH|SMBus' } |
  Where-Object { $_.DeviceID -match 'VEN_1022|VEN_8086' } |
  Select-Object -First 40 Name, DeviceID, Status |
  Format-List | Out-String -Width 200
"#,
        ])
        .output();
    match out {
        Ok(o) => {
            eprintln!("{}", String::from_utf8_lossy(&o.stdout));
            if !o.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&o.stderr));
            }
        }
        Err(e) => eprintln!("ERR CIM PCI dump: {e}"),
    }
    eprintln!("=== PCI 0:14.3 / 0:1F.0 style DeviceIDs ===");
    let out2 = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
Get-PnpDevice -PresentOnly -ErrorAction SilentlyContinue |
  Where-Object { $_.InstanceId -match 'PCI\\VEN_1022&DEV_|PCI\\VEN_8086&DEV_' } |
  Where-Object { $_.InstanceId -match 'CC_0601|BUS_\d{4}&DEV_0014|BUS_\d{4}&DEV_001F|LPC|ISA' -or $_.FriendlyName -match 'LPC|ISA|FCH' } |
  Select-Object -First 30 FriendlyName, InstanceId, Status |
  Format-List | Out-String -Width 220
"#,
        ])
        .output();
    match out2 {
        Ok(o) => eprintln!("{}", String::from_utf8_lossy(&o.stdout)),
        Err(e) => eprintln!("ERR PnP dump: {e}"),
    }
}

fn stage_ecio(dll: &Path, lpc_blob: &[u8]) {
    eprintln!("=== ECIO Gigabyte FW (stage) ===");
    let _isa = match crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5)) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ERR ISA mutex: {e}");
            return;
        }
    };
    match PawnIo::open(dll, lpc_blob) {
        Ok(pio) => {
            let note = crate::ecio::try_clear_firmware_fans(&pio);
            eprintln!("stage ECIO: {note}");
        }
        Err(e) => eprintln!("ERR ECIO open: {e}"),
    }
}

fn stage_summary_isa(dll: &Path, isa_blob: &[u8]) {
    eprintln!("=== stage summary: IsaBridge find + 0x947 ===");
    let _isa = match crate::isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5)) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ERR ISA mutex: {e}");
            return;
        }
    };
    let Ok(pio) = PawnIo::open(dll, isa_blob) else {
        eprintln!("ERR IsaBridge open");
        return;
    };
    let _ = pio.execute("ioctl_map_superio_mmio", &[], 0);
    let _ = pio.execute("ioctl_iomem_mmio_set_state", &[2u64], 0);
    match pio.execute("ioctl_find_superio_mmio", &[], 6) {
        Ok(out) => {
            eprintln!(
                "stage IsaBridge find: [{}]",
                out.iter()
                    .map(|x| format!("0x{x:X}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            for slot in 0u8..=1 {
                let i = slot as usize * 3;
                if out.len() < i + 3 {
                    break;
                }
                let base = out[i];
                let id = out[i + 2] as u16;
                if base == 0 {
                    eprintln!("stage 0x947 slot {slot}: no MMIO base (id=0x{id:04x})");
                    continue;
                }
                match pio.execute(
                    "ioctl_access_superio_mmio",
                    &[u64::from(slot), 0x947, 1, 0, 0],
                    1,
                ) {
                    Ok(v) => eprintln!(
                        "stage 0x947 slot {slot}: 0x{:02X} (id=0x{id:04x})",
                        v.first().copied().unwrap_or(0)
                    ),
                    Err(e) => eprintln!("stage 0x947 slot {slot}: {e}"),
                }
            }
        }
        Err(e) => eprintln!("stage IsaBridge find: {e}"),
    }
}

pub fn run(dll: &Path, lpc_blob: &[u8], isa_blob: Option<&[u8]>) -> Result<(), String> {
    eprintln!("chromaflow-lpc --probe-debug");
    eprintln!("PawnIOLib: {}", dll.display());
    eprintln!("=== stages: LpcIO → IsaBridge/0x947 → ECIO ===");
    dump_pci_bridges();
    // LpcIO before IsaBridge — IsaBridge ITE enter+exit can leave 0x2E stuck.
    if let Err(e) = try_lpc(dll, lpc_blob) {
        eprintln!("ERR LpcIO session: {e}");
    }
    match isa_blob {
        Some(b) => {
            if let Err(e) = try_isa(dll, b) {
                eprintln!("ERR IsaBridgeEC session: {e}");
            }
            stage_summary_isa(dll, b);
        }
        None => eprintln!("IsaBridgeEC.bin not loaded"),
    }
    stage_ecio(dll, lpc_blob);
    eprintln!("=== done ===");
    Ok(())
}
