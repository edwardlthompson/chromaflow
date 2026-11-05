//! JSON-line protocol shared with chromaflow-core (keep fields in sync).

use serde::{Deserialize, Serialize};

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
        #[serde(default = "default_manual")]
        manual: bool,
    },
    Release {
        dir: String,
        pwm: String,
    },
}

fn default_manual() -> bool {
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
    pub ec_base: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorChip {
    pub dir: String,
    pub temps: Vec<Labeled>,
    pub fans: Vec<Labeled>,
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
    pub enable: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ok {
        ok: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        chips: Option<Vec<ChipInfo>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sensors: Option<Vec<SensorChip>>,
    },
    Err {
        ok: bool,
        error: String,
    },
}

impl Response {
    pub fn ok_ping() -> Self {
        Self::Ok {
            ok: true,
            detail: Some("pong".into()),
            chips: None,
            sensors: None,
        }
    }

    pub fn ok_probe(chips: Vec<ChipInfo>) -> Self {
        Self::Ok {
            ok: true,
            detail: None,
            chips: Some(chips),
            sensors: None,
        }
    }

    pub fn ok_sensors(sensors: Vec<SensorChip>) -> Self {
        Self::Ok {
            ok: true,
            detail: None,
            chips: None,
            sensors: Some(sensors),
        }
    }

    pub fn ok_msg(detail: impl Into<String>) -> Self {
        Self::Ok {
            ok: true,
            detail: Some(detail.into()),
            chips: None,
            sensors: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self::Err {
            ok: false,
            error: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock pipe server (no PawnIO): ping / probe / pwm_apply JSON lines.
    fn mock_handle(line: &str) -> Response {
        let req: Request = serde_json::from_str(line).expect("req");
        match req {
            Request::Ping => Response::ok_ping(),
            Request::Probe => Response::ok_probe(vec![ChipInfo {
                name: "it8689".into(),
                chip_id: 0x8689,
                slot: 0,
                pwm_count: 5,
                fan_count: 5,
                hwmon_dir: "hwmon0".into(),
                ec_base: 0xa00,
            }]),
            Request::Sensors => Response::ok_sensors(vec![SensorChip {
                dir: "hwmon0".into(),
                temps: vec![Labeled {
                    label: "CPU".into(),
                    value: "45000".into(),
                }],
                fans: vec![Labeled {
                    label: "CPU_FAN".into(),
                    value: "1200".into(),
                }],
                pwms: vec![PwmReading {
                    name: "pwm1".into(),
                    duty: 128,
                    enable: 1,
                }],
            }]),
            Request::PwmApply { duty, .. } => {
                if duty == 0 {
                    Response::err("0% needs an explicit confirm")
                } else {
                    Response::ok_msg(format!("duty={duty}"))
                }
            }
            Request::Release { .. } => Response::ok_msg("released"),
        }
    }

    #[test]
    fn mock_pipe_ping_probe_set() {
        let ping = mock_handle(r#"{"cmd":"ping"}"#);
        let s = serde_json::to_string(&ping).unwrap();
        assert!(s.contains(r#""ok":true"#));

        let probe = mock_handle(r#"{"cmd":"probe"}"#);
        match probe {
            Response::Ok {
                chips: Some(c), ..
            } => {
                assert_eq!(c.len(), 1);
                assert_eq!(c[0].name, "it8689");
            }
            _ => panic!("probe"),
        }

        let set = mock_handle(
            r#"{"cmd":"pwm_apply","dir":"hwmon0","pwm":"pwm1","duty":200,"manual":true}"#,
        );
        assert!(matches!(set, Response::Ok { ok: true, .. }));

        let zero = mock_handle(
            r#"{"cmd":"pwm_apply","dir":"hwmon0","pwm":"pwm1","duty":0,"manual":true}"#,
        );
        assert!(matches!(zero, Response::Err { .. }));
    }
}
