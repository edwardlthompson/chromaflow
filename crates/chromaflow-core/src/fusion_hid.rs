//! Persistent Fusion HID child (fusion-hid.py serve). No PWM. No OpenRGB C++.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

struct Serve {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
}

static SERVE: Mutex<Option<Serve>> = Mutex::new(None);

pub fn set(kind: &str, hex: &str) -> Result<String, String> {
    if kind != "uniform" && kind != "digital" && kind != "soft" {
        return Err("Fusion HID kind".into());
    }
    valid(hex)?;
    let mut slot = SERVE.lock().unwrap_or_else(|p| p.into_inner());
    if ping(&mut slot, kind, hex).is_err() {
        stop(&mut slot);
        if start(&mut slot).is_ok() && ping(&mut slot, kind, hex).is_ok() {
            return Ok(format!("set Fusion {kind} {hex}"));
        }
        stop(&mut slot);
        return oneshot(kind, hex);
    }
    Ok(format!("set Fusion {kind} {hex}"))
}

fn ping(slot: &mut Option<Serve>, kind: &str, hex: &str) -> Result<(), String> {
    let s = slot.as_mut().ok_or_else(|| "Fusion HID down".to_string())?;
    writeln!(s.stdin, "{kind} {hex}").map_err(|e| e.to_string())?;
    s.stdin.flush().map_err(|e| e.to_string())?;
    match s.rx.recv_timeout(Duration::from_millis(400)) {
        Ok(line) if line.starts_with("ok") => Ok(()),
        Ok(line) => Err(line),
        Err(_) => Err("Fusion HID timeout".into()),
    }
}

fn start(slot: &mut Option<Serve>) -> Result<(), String> {
    let script = script().ok_or_else(|| "fusion-hid.py missing".to_string())?;
    let mut child = Command::new("python3")
        .args(["-u", &script, "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Fusion HID stdin".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Fusion HID stdout".to_string())?;
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("fusion-hid".into())
        .spawn(move || {
            let mut lines = BufReader::new(stdout);
            let mut line = String::new();
            while lines.read_line(&mut line).is_ok() {
                if line.is_empty() {
                    break;
                }
                if tx.send(line.clone()).is_err() {
                    break;
                }
                line.clear();
            }
        })
        .map_err(|e| e.to_string())?;
    *slot = Some(Serve { child, stdin, rx });
    Ok(())
}

fn stop(slot: &mut Option<Serve>) {
    if let Some(mut s) = slot.take() {
        let _ = s.child.kill();
        let _ = s.child.wait();
    }
}

fn oneshot(kind: &str, hex: &str) -> Result<String, String> {
    let script = script().ok_or_else(|| "fusion-hid.py missing".to_string())?;
    let out = Command::new("timeout")
        .args(["0.4", "python3", "-u", &script, kind, hex])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn script() -> Option<String> {
    if let Ok(p) = std::env::var("CHROMAFLOW_FUSION_HID") {
        return Some(p);
    }
    let rel = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/fusion-hid.py");
    if rel.is_file() {
        return Some(rel.to_string_lossy().into());
    }
    let usr = PathBuf::from("/usr/share/chromaflow/fusion-hid.py");
    usr.is_file().then(|| usr.to_string_lossy().into())
}

fn valid(hex: &str) -> Result<(), String> {
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("color must be RRGGBB".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_bad_kind_and_hex() {
        assert!(super::set("sync", "00FF00").is_err());
        assert!(super::set("uniform", "").is_err());
        assert!(super::set("uniform", "00").is_err());
        assert!(super::valid("00FFAA").is_ok());
    }
}
