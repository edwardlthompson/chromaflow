//! Elevated SuperIO LPC helper (ADR-0022). Windows only.

mod proto;

#[cfg(windows)]
mod ecio;
#[cfg(windows)]
mod isa_bridge;
#[cfg(windows)]
mod isa_mutex;
#[cfg(windows)]
mod ite;
#[cfg(windows)]
mod pawnio;
#[cfg(windows)]
mod probe_debug;
#[cfg(windows)]
mod server;

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("chromaflow-lpc is Windows-only");
        std::process::exit(2);
    }
    #[cfg(windows)]
    {
        if let Err(e) = run() {
            eprintln!("chromaflow-lpc: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!(
            "Usage:\n\
             chromaflow-lpc --serve         Elevated named-pipe SuperIO bridge\n\
             chromaflow-lpc --probe-debug   One-shot LpcIO/IsaBridgeEC/PCI dump"
        );
        return Ok(());
    }

    let dll = std::path::PathBuf::from(r"C:\Program Files\PawnIO\PawnIOLib.dll");
    if !dll.is_file() {
        return Err(format!(
            "PawnIOLib.dll missing at {} — install namazso.PawnIO",
            dll.display()
        ));
    }

    if args.iter().any(|a| a == "--probe-debug") {
        let lpc_blob = load_module_bin("LpcIO.bin")?;
        let isa_blob = load_module_bin("IsaBridgeEC.bin").ok();
        return probe_debug::run(&dll, &lpc_blob, isa_blob.as_deref());
    }

    if args.iter().any(|a| a == "--pre-read") {
        let lpc_blob = load_module_bin("LpcIO.bin")?;
        return pre_read_only(&dll, &lpc_blob);
    }

    if !args.iter().any(|a| a == "--serve") {
        eprintln!("chromaflow-lpc requires --serve, --probe-debug, or --pre-read");
        std::process::exit(2);
    }

    let lpc_blob = load_module_bin("LpcIO.bin")?;
    let lpc = pawnio::PawnIo::open(&dll, &lpc_blob)?;
    let isa = match load_module_bin("IsaBridgeEC.bin") {
        Ok(blob) => match pawnio::PawnIo::open(&dll, &blob) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("chromaflow-lpc: IsaBridgeEC open failed: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("chromaflow-lpc: IsaBridgeEC.bin missing ({e}) — LpcIO only");
            None
        }
    };
    server::run_serve(lpc, isa)
}

/// Read SuperIO chip IDs and try alternate unlock sequences (bring-up only).
#[cfg(windows)]
fn pre_read_only(dll: &std::path::Path, lpc_blob: &[u8]) -> Result<(), String> {
    let _isa = isa_mutex::IsaBusGuard::acquire(std::time::Duration::from_secs(5))?;
    let pio = pawnio::PawnIo::open(dll, lpc_blob)?;
    for slot in [1u8, 0u8] {
        pio.select_slot(slot)?;
        let id = pio.sio_inw(0x20).unwrap_or(0xffff);
        let b20 = pio.sio_inb(0x20).unwrap_or(0xff);
        let b21 = pio.sio_inb(0x21).unwrap_or(0xff);
        let _ = pio.sio_outb(0x07, 0x04);
        let bh = pio.sio_inb(0x60).unwrap_or(0xff);
        let bl = pio.sio_inb(0x61).unwrap_or(0xff);
        eprintln!(
            "pre-read slot {slot}: id=0x{id:04x} [0x{b20:02x} 0x{b21:02x}] LDN4=0x{bh:02x}{bl:02x}"
        );
    }
    // Probe common Gigabyte EC windows (needs LpcIO allowlist / FindBars).
    let _ = pio.select_slot(0);
    for &base in &[0x0a20u16, 0x0a21, 0x0a40, 0x0a60, 0x0290, 0x025c] {
        match pio.pio_inb(base) {
            Ok(v) => eprintln!("pio_inb 0x{base:04x} = 0x{v:02x}"),
            Err(e) => eprintln!("pio_inb 0x{base:04x}: {e}"),
        }
    }

    let try_seq = |pio: &pawnio::PawnIo, slot: u8, label: &str, seq: &[u8]| {
        let idx = if slot == 0 { 0x2eu16 } else { 0x4eu16 };
        let _ = pio.select_slot(slot);
        for &b in seq {
            let _ = pio.pio_outb(idx, b);
            std::thread::sleep(std::time::Duration::from_micros(50));
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id = pio.sio_inw(0x20).unwrap_or(0xffff);
        eprintln!("try {label} slot {slot}: id=0x{id:04x}");
        id
    };

    // Exit both first (reset half-open config), then secondary-first ITE enter.
    for slot in [0u8, 1u8] {
        let idx = if slot == 0 { 0x2eu16 } else { 0x4eu16 };
        let _ = pio.select_slot(slot);
        let _ = pio.pio_outb(idx, 0x02);
        let _ = pio.pio_outb(idx.wrapping_add(1), 0x02);
    }
    std::thread::sleep(std::time::Duration::from_millis(10));

    let s1 = try_seq(&pio, 1, "ITE", &[0x87, 0x01, 0x55, 0xaa]);
    let s0 = try_seq(&pio, 0, "ITE-after-sec", &[0x87, 0x01, 0x55, 0x55]);
    let _ = try_seq(&pio, 1, "Winbond", &[0x87, 0x87]);
    let _ = try_seq(&pio, 1, "SMSC", &[0x55]);
    let _ = try_seq(&pio, 1, "ITE-x2", &[0x87, 0x01, 0x55, 0xaa, 0x87, 0x01, 0x55, 0xaa]);
    // If either chip answered, dump LDN4 base.
    for (slot, id) in [(1u8, s1), (0u8, s0)] {
        if id != 0 && id != 0xffff {
            let _ = pio.select_slot(slot);
            let _ = pio.sio_outb(0x07, 0x04);
            let bh = pio.sio_inb(0x60).unwrap_or(0xff);
            let bl = pio.sio_inb(0x61).unwrap_or(0xff);
            eprintln!("live slot {slot} id=0x{id:04x} EC base regs=0x{bh:02x}{bl:02x}");
            let _ = pio.find_bars();
            for &base in &[0x0a20u16, 0x0a40, 0x0a60, 0x0290] {
                match pio.pio_inb(base) {
                    Ok(v) => eprintln!("  pio_inb 0x{base:04x}=0x{v:02x}"),
                    Err(e) => eprintln!("  pio_inb 0x{base:04x}: {e}"),
                }
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn load_module_bin(name: &str) -> Result<Vec<u8>, String> {
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(name))),
        Some(std::path::PathBuf::from(r"C:\Program Files\ChromaFlow").join(name)),
        Some(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("modules")
                .join(name),
        ),
    ];
    for path in candidates.into_iter().flatten() {
        if path.is_file() {
            return std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()));
        }
    }
    Err(format!(
        "{name} not found next to exe, Program Files\\ChromaFlow, or crate modules/"
    ))
}
