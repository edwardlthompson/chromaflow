//! Unelevated named-pipe client for chromaflow-lpc (ADR-0022).

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

pub const PIPE_NAME: &str = r"\\.\pipe\chromaflow-lpc";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Ping,
    Probe,
    Sensors,
    PwmApply {
        dir: String,
        pwm: String,
        duty: u8,
        #[serde(default = "default_true")]
        manual: bool,
    },
    Release {
        dir: String,
        pwm: String,
    },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipInfo {
    pub name: String,
    pub chip_id: u16,
    pub slot: u8,
    pub pwm_count: u8,
    pub fan_count: u8,
    pub hwmon_dir: String,
    #[serde(default)]
    pub ec_base: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorChip {
    pub dir: String,
    #[serde(default)]
    pub temps: Vec<Labeled>,
    #[serde(default)]
    pub fans: Vec<Labeled>,
    #[serde(default)]
    pub pwms: Vec<PwmReading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Labeled {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmReading {
    pub name: String,
    pub duty: u8,
    #[serde(default)]
    pub enable: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub chips: Option<Vec<ChipInfo>>,
    #[serde(default)]
    pub sensors: Option<Vec<SensorChip>>,
}

static CLIENT: Mutex<()> = Mutex::new(());

fn pipe_path() -> String {
    std::env::var("CHROMAFLOW_LPC_PIPE").unwrap_or_else(|_| PIPE_NAME.into())
}

#[cfg(windows)]
fn open_pipe() -> Result<std::fs::File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    let path = pipe_path();
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&path)
        .map_err(|e| format!("open {path}: {e}"))
}

#[cfg(not(windows))]
fn open_pipe() -> Result<std::fs::File, String> {
    Err("LPC pipe is Windows-only".into())
}

fn roundtrip(req: &Request) -> Result<Response, String> {
    let _guard = CLIENT.lock().map_err(|_| "lpc client lock")?;
    let mut file = open_pipe()?;
    let line = serde_json::to_string(req).map_err(|e| e.to_string())? + "\n";
    file.write_all(line.as_bytes())
        .map_err(|e| format!("pipe write: {e}"))?;
    file.flush().map_err(|e| format!("pipe flush: {e}"))?;
    let mut reader = BufReader::new(file.try_clone().map_err(|e| e.to_string())?);
    let mut resp_line = String::new();
    reader
        .read_line(&mut resp_line)
        .map_err(|e| format!("pipe read: {e}"))?;
    drop(file);
    serde_json::from_str(resp_line.trim()).map_err(|e| format!("pipe json: {e}"))
}

pub fn ping() -> Result<(), String> {
    let r = roundtrip(&Request::Ping)?;
    if r.ok {
        Ok(())
    } else {
        Err(r.error.unwrap_or_else(|| "ping failed".into()))
    }
}

pub fn pipe_up() -> bool {
    ping().is_ok()
}

pub fn probe() -> Result<Vec<ChipInfo>, String> {
    let r = roundtrip(&Request::Probe)?;
    if !r.ok {
        return Err(r.error.unwrap_or_else(|| "probe failed".into()));
    }
    Ok(r.chips.unwrap_or_default())
}

pub fn sensors() -> Result<Vec<SensorChip>, String> {
    let r = roundtrip(&Request::Sensors)?;
    if !r.ok {
        return Err(r.error.unwrap_or_else(|| "sensors failed".into()));
    }
    Ok(r.sensors.unwrap_or_default())
}

pub fn pipe_duty(dir: &str, pwm: &str, duty: u8) -> Result<(), String> {
    let r = roundtrip(&Request::PwmApply {
        dir: dir.into(),
        pwm: pwm.into(),
        duty,
        manual: true,
    })?;
    if r.ok {
        Ok(())
    } else {
        Err(r.error.unwrap_or_else(|| "pipe duty failed".into()))
    }
}

pub fn release(dir: &str, pwm: &str) -> Result<(), String> {
    let r = roundtrip(&Request::Release {
        dir: dir.into(),
        pwm: pwm.into(),
    })?;
    if r.ok {
        Ok(())
    } else {
        Err(r.error.unwrap_or_else(|| "release failed".into()))
    }
}

/// Materialize helper sensor readings into a Linux-shaped hwmon tree.
pub fn write_shadow(root: &std::path::Path, sensors: &[SensorChip]) -> Result<(), String> {
    use std::fs;
    for chip in sensors {
        let dir = root.join(&chip.dir);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        // Name file left to probe materialize; keep if present.
        for (i, t) in chip.temps.iter().enumerate() {
            let n = i + 1;
            fs::write(dir.join(format!("temp{n}_input")), format!("{}\n", t.value))
                .map_err(|e| e.to_string())?;
            fs::write(dir.join(format!("temp{n}_label")), format!("{}\n", t.label))
                .map_err(|e| e.to_string())?;
        }
        for (i, f) in chip.fans.iter().enumerate() {
            let n = i + 1;
            fs::write(dir.join(format!("fan{n}_input")), format!("{}\n", f.value))
                .map_err(|e| e.to_string())?;
            fs::write(dir.join(format!("fan{n}_label")), format!("{}\n", f.label))
                .map_err(|e| e.to_string())?;
        }
        for p in &chip.pwms {
            fs::write(dir.join(&p.name), format!("{}\n", p.duty)).map_err(|e| e.to_string())?;
            fs::write(
                dir.join(format!("{}_enable", p.name)),
                format!("{}\n", if p.enable == 0 { 2 } else { p.enable }),
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn materialize_from_probe(root: &std::path::Path, chips: &[ChipInfo]) -> Result<(), String> {
    use std::fs;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    for c in chips {
        let dir = root.join(&c.hwmon_dir);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(dir.join("name"), format!("{}\n", c.name)).map_err(|e| e.to_string())?;
        for i in 1..=c.pwm_count {
            let pwm = format!("pwm{i}");
            if !dir.join(&pwm).is_file() {
                fs::write(dir.join(&pwm), "128\n").map_err(|e| e.to_string())?;
                fs::write(dir.join(format!("{pwm}_enable")), "2\n").map_err(|e| e.to_string())?;
            }
        }
        for i in 1..=c.fan_count {
            let fan = format!("fan{i}_input");
            if !dir.join(&fan).is_file() {
                fs::write(dir.join(&fan), "0\n").map_err(|e| e.to_string())?;
                fs::write(dir.join(format!("fan{i}_label")), format!("fan{i}\n"))
                    .map_err(|e| e.to_string())?;
            }
        }
        for i in 1..=3u8 {
            let t = format!("temp{i}_input");
            if !dir.join(&t).is_file() {
                fs::write(dir.join(&t), "40000\n").map_err(|e| e.to_string())?;
                fs::write(dir.join(format!("temp{i}_label")), format!("temp{i}\n"))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ping_json() {
        let s = serde_json::to_string(&Request::Ping).unwrap();
        assert!(s.contains("ping"));
        assert!(s.contains(r#""cmd":"ping""#));
    }

    #[test]
    fn request_pwm_apply_json() {
        let s = serde_json::to_string(&Request::PwmApply {
            dir: "hwmon0".into(),
            pwm: "pwm1".into(),
            duty: 200,
            manual: true,
        })
        .unwrap();
        assert!(s.contains("pwm_apply"));
        assert!(s.contains("200"));
    }

    #[test]
    fn response_ok_and_err() {
        let ok: Response = serde_json::from_str(r#"{"ok":true}"#).unwrap();
        assert!(ok.ok);
        let err: Response =
            serde_json::from_str(r#"{"ok":false,"error":"nope"}"#).unwrap();
        assert!(!err.ok);
        assert_eq!(err.error.as_deref(), Some("nope"));
    }
}
