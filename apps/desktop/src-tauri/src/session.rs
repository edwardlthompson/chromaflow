//! Persist last tab and per-device host effects. No PWM.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;

const TABS: [&str; 4] = ["Cooling", "Lighting", "Profiles", "Support"];
const MAX: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub tab: String,
    #[serde(default)]
    pub last_mode: Map<String, Value>,
    #[serde(default)]
    pub last_color: Map<String, Value>,
    #[serde(default)]
    pub effect_speed: u32,
    #[serde(default)]
    pub gauge_metric: String,
    #[serde(default)]
    pub gauge_palette: String,
}

fn path() -> std::path::PathBuf {
    chromaflow_core::profiles::config_dir().join("session.json")
}

fn clean_map(raw: &Map<String, Value>, color: bool) -> Map<String, Value> {
    let mut out = Map::new();
    for (k, v) in raw {
        if out.len() >= MAX {
            break;
        }
        let key = k.chars().take(120).collect::<String>();
        let Some(val) = v.as_str() else { continue };
        let val = val.chars().take(80).collect::<String>();
        if key.is_empty() || val.is_empty() {
            continue;
        }
        let mut hex = val.len() == 7 && val.starts_with('#');
        hex = hex && val.as_bytes()[1..].iter().all(|b| b.is_ascii_hexdigit());
        if color && !hex {
            continue;
        }
        out.insert(key, Value::String(val));
    }
    out
}

pub fn sanitize(mut s: Session) -> Session {
    if !TABS.contains(&s.tab.as_str()) {
        s.tab = "Cooling".into();
    }
    s.last_mode = clean_map(&s.last_mode, false);
    s.last_color = clean_map(&s.last_color, true);
    s.effect_speed = if (1..=255).contains(&s.effect_speed) {
        s.effect_speed
    } else {
        128
    };
    s.gauge_metric = if s.gauge_metric == "usage" {
        "usage".into()
    } else {
        "temp".into()
    };
    s.gauge_palette = if s.gauge_palette == "green" {
        "green".into()
    } else {
        "blue".into()
    };
    s
}

pub fn load() -> Session {
    let Ok(raw) = fs::read_to_string(path()) else {
        return sanitize(Session::default());
    };
    sanitize(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save(s: Session) -> Result<Session, String> {
    let s = sanitize(s);
    let dir = chromaflow_core::profiles::config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
    fs::write(path(), body).map_err(|e| e.to_string())?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_tab_color_and_speed() {
        let mut s = Session {
            tab: "Nope".into(),
            effect_speed: 900,
            gauge_metric: "hot".into(),
            ..Session::default()
        };
        s.last_color.insert("k".into(), Value::String("red".into()));
        s.last_color
            .insert("openrgb:K".into(), Value::String("#00ff00".into()));
        s.last_mode
            .insert("openrgb:K".into(), Value::String("Rainbow Wave".into()));
        let s = sanitize(s);
        assert_eq!(s.tab, "Cooling");
        assert_eq!(s.effect_speed, 255);
        assert_eq!(s.gauge_metric, "temp");
        assert_eq!(s.gauge_palette, "blue");
        assert_eq!(s.last_color.len(), 1);
        assert_eq!(
            s.last_mode.get("openrgb:K").and_then(Value::as_str),
            Some("Rainbow Wave")
        );
    }
}
