//! LPC / SuperIO IO provider. Real port IO lives in an elevated helper (ADR-0022).
//! This module speaks a tiny named-pipe protocol or a fixture mock for CI.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

const PIPE_ENV: &str = "CHROMAFLOW_LPC_PIPE";
const MOCK_ENV: &str = "CHROMAFLOW_LPC_MOCK";
static JITTER_STARTED: AtomicBool = AtomicBool::new(false);
static JITTER_BASE: Mutex<Vec<(PathBuf, i32)>> = Mutex::new(Vec::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoKind {
    Unavailable,
    Mock,
    Helper,
}

pub fn kind() -> IoKind {
    // Explicit lab only — never auto-mock from a leftover fixture (that made
    // Auto-calibrate write files while real fans never moved).
    if std::env::var(MOCK_ENV).is_ok() || lab_enabled() {
        return IoKind::Mock;
    }
    if std::env::var(PIPE_ENV).is_ok() || helper_bridge_present() {
        return IoKind::Helper;
    }
    IoKind::Unavailable
}

/// Support → Apply / lab scripts write this so file PWM works without a process env var.
fn lab_enabled() -> bool {
    mock_root().join(".lab").is_file()
}

pub fn enable_lab_fixture() -> PathBuf {
    let root = ensure_mock_fixture();
    let _ = fs::write(root.join(".lab"), "1\n");
    root
}

pub fn disable_lab_fixture() {
    let _ = fs::remove_file(mock_root().join(".lab"));
}

/// Official PawnIO driver under Program Files (namazso.PawnIO / winget).
pub fn pawnio_installed() -> bool {
    Path::new(r"C:\Program Files\PawnIO\PawnIO.sys").is_file()
        || Path::new(r"C:\Program Files\PawnIO\PawnIOLib.dll").is_file()
}

pub fn chromaflowd_installed() -> bool {
    Path::new(r"C:\Program Files\ChromaFlow\chromaflowd.exe").is_file()
}

/// True only when the elevated SuperIO↔PawnIO bridge binary (or pipe override) exists.
/// `chromaflowd.exe` alone is the watchdog — it is not LPC port IO.
fn helper_bridge_present() -> bool {
    Path::new(r"C:\Program Files\ChromaFlow\chromaflow-lpc.exe").is_file()
}

pub fn superio_bridge_ready() -> bool {
    helper_bridge_present() || std::env::var(PIPE_ENV).is_ok()
}

/// True when the elevated helper answers ping on the named pipe.
pub fn bridge_live() -> bool {
    crate::windows::lpc_client::pipe_up()
}

/// Shadow hwmon tree mirrored from chromaflow-lpc sensors.
pub fn live_root() -> PathBuf {
    crate::paths::runtime_dir().join("lpc-live")
}

/// Ensure helper is reachable; try starting the scheduled task once if binary exists.
pub fn ensure_helper() -> Result<(), String> {
    if crate::windows::lpc_client::pipe_up() {
        return Ok(());
    }
    if !helper_bridge_present() {
        return Err("chromaflow-lpc.exe not installed under Program Files\\ChromaFlow".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("schtasks")
            .args(["/Run", "/TN", "ChromaFlowLPC"])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .status();
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(100));
            if crate::windows::lpc_client::pipe_up() {
                return Ok(());
            }
        }
    }
    Err("chromaflow-lpc pipe not up (is the ChromaFlowLPC task / service running?)".into())
}

pub fn mock_root() -> PathBuf {
    if let Ok(p) = std::env::var(MOCK_ENV) {
        if !p.is_empty() && p != "1" {
            return PathBuf::from(p);
        }
    }
    crate::paths::runtime_dir().join("lpc-mock")
}

/// Seed a hwmon-shaped fixture so Cooling UI + pwm_apply can be exercised on Windows CI.
const FIXTURE_VER: &str = "3-x570s";

pub fn ensure_mock_fixture() -> PathBuf {
    let root = mock_root();
    let ver = root.join(".fixture-ver");
    let current = fs::read_to_string(&ver).unwrap_or_default();
    if current.trim() == FIXTURE_VER && root.join("hwmon0").join("name").is_file() {
        start_temp_jitter(&root);
        return root;
    }
    // Gigabyte X570S AORUS MASTER dual-ITE map (matches coolingBoard.js names).
    write_ite_chip(
        &root.join("hwmon0"),
        "it8689",
        &[
            ("CPU", "45000"),
            ("DIMM", "38000"),
            ("System", "40000"),
        ],
        &[
            ("CPU_FAN", "1200"),
            ("SYS_FAN1", "900"),
            ("SYS_FAN2", "850"),
            ("SYS_FAN3", "800"),
            ("CPU_OPT", "1100"),
        ],
        5,
    );
    write_ite_chip(
        &root.join("hwmon1"),
        "it87952",
        &[("VRM", "52000"), ("NVMe", "45000")],
        &[
            ("FAN4", "700"),
            ("FAN5_PUMP", "2400"),
            ("FAN6_PUMP", "2300"),
            ("FAN7_PUMP", "2200"),
            ("FAN8", "650"),
        ],
        5,
    );
    let _ = fs::write(&ver, FIXTURE_VER);
    start_temp_jitter(&root);
    root
}

/// Tiny ±0.5–1.5 °C drift on mock board temps so lab sparks are not perfectly flat.
fn start_temp_jitter(root: &Path) {
    let mut bases = Vec::new();
    for entry in fs::read_dir(root).into_iter().flatten().flatten() {
        let chip = entry.path();
        if !chip.is_dir() {
            continue;
        }
        for i in 1..=8u8 {
            let path = chip.join(format!("temp{i}_input"));
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(v) = raw.trim().parse::<i32>() {
                    bases.push((path, v));
                }
            }
        }
    }
    if bases.is_empty() {
        return;
    }
    if let Ok(mut g) = JITTER_BASE.lock() {
        *g = bases;
    }
    if JITTER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::Builder::new()
        .name("cf-lpc-jitter".into())
        .spawn(|| {
            let t0 = Instant::now();
            loop {
                thread::sleep(Duration::from_secs(2));
                let Ok(bases) = JITTER_BASE.lock() else {
                    continue;
                };
                let phase = t0.elapsed().as_secs_f64();
                for (i, (path, base)) in bases.iter().enumerate() {
                    // ±1000..1500 m°C (~1–1.5 °C), slow sine — not enough to thrash PWM.
                    let amp = 1000.0 + (i as f64 % 3.0) * 250.0;
                    let delta = (phase * 0.35 + i as f64).sin() * amp;
                    let v = (*base as f64 + delta).round() as i32;
                    let _ = fs::write(path, format!("{v}\n"));
                }
            }
        })
        .ok();
}

fn write_ite_chip(
    chip: &Path,
    name: &str,
    temps: &[(&str, &str)],
    fans: &[(&str, &str)],
    pwm_n: u8,
) {
    let _ = fs::create_dir_all(chip);
    let _ = fs::write(chip.join("name"), format!("{name}\n"));
    for (i, (label, val)) in temps.iter().enumerate() {
        let n = i + 1;
        let _ = fs::write(chip.join(format!("temp{n}_input")), format!("{val}\n"));
        let _ = fs::write(chip.join(format!("temp{n}_label")), format!("{label}\n"));
    }
    for (i, (label, rpm)) in fans.iter().enumerate() {
        let n = i + 1;
        let _ = fs::write(chip.join(format!("fan{n}_input")), format!("{rpm}\n"));
        let _ = fs::write(chip.join(format!("fan{n}_label")), format!("{label}\n"));
    }
    for i in 1..=pwm_n {
        let duty = 80 + i * 10;
        let _ = fs::write(chip.join(format!("pwm{i}")), format!("{duty}\n"));
        let _ = fs::write(chip.join(format!("pwm{i}_enable")), "2\n");
    }
}

/// Read an 8-bit LPC register via helper or mock (mock returns 0).
pub fn inb(_port: u16) -> Result<u8, String> {
    match kind() {
        IoKind::Mock => Ok(0),
        IoKind::Unavailable => Err("LPC helper unavailable (install ChromaFlow service / PawnIO)".into()),
        IoKind::Helper => Err("LPC helper pipe not connected yet".into()),
    }
}

/// Write an 8-bit LPC register via helper or mock.
pub fn outb(_port: u16, _value: u8) -> Result<(), String> {
    match kind() {
        IoKind::Mock => Ok(()),
        IoKind::Unavailable => Err("LPC helper unavailable (install ChromaFlow service / PawnIO)".into()),
        IoKind::Helper => Err("LPC helper pipe not connected yet".into()),
    }
}
