//! Named-pipe JSON-line server for unelevated chromaflow-core.

#![cfg(windows)]

use crate::isa_bridge::{self, MmioChip};
use crate::ite::{self, LiveChip};
use crate::pawnio::PawnIo;
use crate::proto::{ChipInfo, Request, Response, PIPE_NAME};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::io::{FromRawHandle, IntoRawHandle, RawHandle};
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
    PIPE_WAIT,
};

enum Backend {
    Lpc {
        pio: PawnIo,
        chips: Vec<LiveChip>,
    },
    Isa {
        pio: PawnIo,
        chips: Vec<MmioChip>,
    },
}

impl Backend {
    fn probe(&mut self) -> Result<Vec<ChipInfo>, String> {
        match self {
            Backend::Lpc { pio, chips } => {
                let found = ite::probe(pio)?;
                *chips = found.clone();
                Ok(found.into_iter().map(|c| c.info).collect())
            }
            Backend::Isa { pio, chips } => match isa_bridge::probe(pio) {
                Ok(found) => {
                    *chips = found.clone();
                    Ok(found.into_iter().map(|c| c.info).collect())
                }
                Err(e) => {
                    // Keep prior chips if re-probe fails mid-session.
                    if chips.is_empty() {
                        Ok(Vec::new())
                    } else {
                        Err(e)
                    }
                }
            },
        }
    }

    fn ensure_chips(&mut self) {
        match self {
            Backend::Lpc { pio, chips } if chips.is_empty() => {
                if let Ok(found) = ite::probe(pio) {
                    *chips = found;
                }
            }
            Backend::Isa { pio, chips } if chips.is_empty() => {
                if let Ok(found) = isa_bridge::probe(pio) {
                    *chips = found;
                }
            }
            _ => {}
        }
    }

    fn sensors_all(&mut self) -> Result<Vec<crate::proto::SensorChip>, String> {
        self.ensure_chips();
        match self {
            Backend::Lpc { pio, chips } => ite::read_sensors(pio, chips),
            Backend::Isa { pio, chips } => isa_bridge::read_sensors(pio, chips),
        }
    }

    fn pwm_apply(&mut self, dir: &str, pwm: &str, duty: u8) -> Result<(), String> {
        self.ensure_chips();
        match self {
            Backend::Lpc { pio, chips } => ite::pwm_apply(pio, chips, dir, pwm, duty, true),
            Backend::Isa { pio, chips } => isa_bridge::pwm_apply(pio, chips, dir, pwm, duty, true),
        }
    }

    fn release(&mut self, dir: &str, pwm: &str) -> Result<(), String> {
        match self {
            Backend::Lpc { pio, chips } => ite::release(pio, chips, dir, pwm),
            Backend::Isa { pio, chips } => isa_bridge::release(pio, chips, dir, pwm),
        }
    }

    fn failsafe(&mut self) {
        match self {
            Backend::Lpc { pio, chips } => ite::release_all(pio, chips),
            Backend::Isa { pio, chips } => isa_bridge::release_all(pio, chips),
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn handle_line(state: &Mutex<Backend>, line: &str) -> String {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => return serde_json::to_string(&Response::err(format!("bad json: {e}"))).unwrap(),
    };
    let mut g = match state.lock() {
        Ok(g) => g,
        Err(_) => return serde_json::to_string(&Response::err("lock")).unwrap(),
    };
    let resp = match req {
        Request::Ping => Response::ok_ping(),
        Request::Probe => match g.probe() {
            Ok(chips) => Response::ok_probe(chips),
            Err(e) => Response::err(e),
        },
        Request::Sensors => match g.sensors_all() {
            Ok(s) => Response::ok_sensors(s),
            Err(e) => Response::err(e),
        },
        Request::PwmApply {
            dir,
            pwm,
            duty,
            manual: _,
        } => match g.pwm_apply(&dir, &pwm, duty) {
            Ok(()) => Response::ok_msg(format!("set {dir}/{pwm}={duty}")),
            Err(e) => Response::err(e),
        },
        Request::Release { dir, pwm } => match g.release(&dir, &pwm) {
            Ok(()) => Response::ok_msg(format!("release {dir}/{pwm}")),
            Err(e) => Response::err(e),
        },
    };
    serde_json::to_string(&resp).unwrap_or_else(|e| {
        serde_json::to_string(&Response::err(e.to_string())).unwrap()
    })
}

fn serve_client(state: &Mutex<Backend>, handle: HANDLE) {
    let raw = handle as RawHandle;
    let file = unsafe { std::fs::File::from_raw_handle(raw) };
    let mut reader = BufReader::new(file.try_clone().expect("clone pipe"));
    let mut writer = file;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let resp = handle_line(state, line.trim());
                let mut out = resp;
                out.push('\n');
                if writer.write_all(out.as_bytes()).is_err() {
                    break;
                }
                let _ = writer.flush();
            }
            Err(_) => break,
        }
    }
    let _ = writer.into_raw_handle();
}

/// Prefer classic LpcIO; fall back to IsaBridgeEC (Gigabyte AMD MMIO).
/// After IsaBridge clears Gigabyte FW latch, retry LpcIO once (FC-like order).
/// Always listens for ping even when probe finds no chips (honest empty inventory).
pub fn run_serve(lpc: PawnIo, isa: Option<PawnIo>) -> Result<(), String> {
    // Prefer classic LpcIO with Gigabyte secondary-first enter inside ite::probe.
    let backend = match ite::probe(&lpc) {
        Ok(chips) if !chips.is_empty() => {
            eprintln!("chromaflow-lpc: LpcIO probed {} chip(s)", chips.len());
            drop(isa);
            Backend::Lpc { pio: lpc, chips }
        }
        other => {
            match &other {
                Err(e) => eprintln!("chromaflow-lpc: LpcIO probe: {e}"),
                Ok(_) => eprintln!("chromaflow-lpc: LpcIO probe: no chips"),
            }
            if let Some(isa_pio) = isa {
                let isa_chips = match isa_bridge::probe(&isa_pio) {
                    Ok(chips) => {
                        eprintln!(
                            "chromaflow-lpc: IsaBridgeEC probed {} chip(s)",
                            chips.len()
                        );
                        chips
                    }
                    Err(e) => {
                        eprintln!("chromaflow-lpc: IsaBridgeEC probe: {e}");
                        Vec::new()
                    }
                };
                // IsaBridge 0xFF / empty → ECIO Gigabyte FW clear, then retry classic ITE.
                if isa_chips.is_empty() {
                    let note = crate::ecio::try_clear_firmware_fans(&lpc);
                    eprintln!("chromaflow-lpc: {note}");
                }
                let lpc_retry = if isa_chips.is_empty() {
                    ite::probe(&lpc).ok().filter(|c| !c.is_empty())
                } else {
                    None
                };
                if let Some(retry) = lpc_retry {
                    eprintln!(
                        "chromaflow-lpc: LpcIO retry after IsaBridge: {} chip(s)",
                        retry.len()
                    );
                    drop(isa_pio);
                    Backend::Lpc {
                        pio: lpc,
                        chips: retry,
                    }
                } else {
                    drop(lpc);
                    Backend::Isa {
                        pio: isa_pio,
                        chips: isa_chips,
                    }
                }
            } else {
                Backend::Lpc {
                    pio: lpc,
                    chips: Vec::new(),
                }
            }
        }
    };

    let state = Mutex::new(backend);
    let name = wide(PIPE_NAME);
    eprintln!("chromaflow-lpc: listening on {PIPE_NAME}");

    loop {
        let handle = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            if let Ok(mut g) = state.lock() {
                g.failsafe();
            }
            return Err(format!(
                "CreateNamedPipeW failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let ok = unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) };
        if ok == 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(535) {
                unsafe {
                    let _ = CloseHandle(handle);
                }
                eprintln!("chromaflow-lpc: connect: {err}");
                continue;
            }
        }
        serve_client(&state, handle);
    }
}
