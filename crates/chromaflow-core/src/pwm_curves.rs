//! Curve set for the PWM watchdog. Never writes sysfs.

use crate::profiles;
use crate::pwm_policy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Channel {
    pub chip: String,
    pub pwm: String,
    pub dir: String,
    #[serde(default)]
    pub source_chip: String,
    #[serde(default)]
    pub source_label: String,
    pub enabled: bool,
    #[serde(default)]
    pub curve_id: String,
    #[serde(default)]
    pub temp_id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub step_up: u8,
    #[serde(default)]
    pub step_down: u8,
    #[serde(default)]
    pub start_pct: u8,
    #[serde(default)]
    pub stop_pct: u8,
    #[serde(default)]
    pub offset: i8,
    #[serde(default)]
    pub min_pct: u8,
    #[serde(default)]
    pub hysteresis_c: u8,
    #[serde(default)]
    pub response_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MixSensor {
    pub id: String,
    pub label: String,
    pub op: String,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub offset: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FanUnit {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamedCurve {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub points: Vec<[f32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveFile {
    pub schema: u32,
    pub allow_zero: bool,
    pub min_duty: u8,
    pub max_duty: u8,
    #[serde(default)]
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub mixes: Vec<MixSensor>,
    #[serde(default)]
    pub names: HashMap<String, String>,
    #[serde(default)]
    pub calibration: HashMap<String, Vec<[u32; 2]>>,
    #[serde(default)]
    pub units: Vec<FanUnit>,
    #[serde(default)]
    pub custom: Vec<NamedCurve>,
    #[serde(default)]
    pub hidden: HashMap<String, bool>,
    #[serde(default)]
    pub shown: HashMap<String, bool>,
}

pub fn empty() -> CurveFile {
    CurveFile {
        schema: 2,
        allow_zero: false,
        min_duty: pwm_policy::MIN_PERCENT,
        max_duty: 100,
        channels: Vec::new(),
        mixes: Vec::new(),
        names: HashMap::new(),
        calibration: HashMap::new(),
        units: Vec::new(),
        custom: Vec::new(),
        hidden: HashMap::new(),
        shown: HashMap::new(),
    }
}

pub fn path() -> PathBuf {
    profiles::config_dir().join("curves.json")
}

fn migrate(mut file: CurveFile) -> Result<CurveFile, String> {
    if file.schema != 1 && file.schema != 2 {
        return Err("unsupported curves schema".into());
    }
    file.schema = 2;
    for ch in &mut file.channels {
        if ch.curve_id.is_empty() {
            ch.curve_id = "balanced".into();
        }
        if ch.min_pct == 0 && !file.allow_zero {
            ch.min_pct = pwm_policy::MIN_PERCENT;
        }
        if ch.kind.is_empty() {
            ch.kind = "fan".into();
        }
        if ch.temp_id.is_empty() && !ch.source_chip.is_empty() {
            ch.temp_id = format!("hwmon:{}/{}", ch.source_chip, ch.source_label);
        }
    }
    Ok(file)
}

pub fn load() -> Result<CurveFile, String> {
    let p = path();
    if !p.is_file() {
        return Ok(empty());
    }
    let file: CurveFile = serde_json::from_str(&fs::read_to_string(&p).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    migrate(file)
}

pub fn save(file: &CurveFile) -> Result<(), String> {
    let file = migrate(file.clone())?;
    let dir = profiles::config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let tmp = dir.join("curves.json.tmp");
    fs::write(
        &tmp,
        serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&tmp, path()).map_err(|e| e.to_string())?;
    let cool = serde_json::json!({
        "schema": 2,
        "names": file.names,
        "mixes": file.mixes,
        "calibration": file.calibration,
        "units": file.units,
        "custom": file.custom,
        "hidden": file.hidden,
        "shown": file.shown,
    });
    fs::write(
        dir.join("cooling.json"),
        serde_json::to_string_pretty(&cool).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
