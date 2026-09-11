use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {
    pub hwmon: Vec<HwmonChip>,
    pub hidraw: Vec<DevNode>,
    pub i2c: Vec<DevNode>,
    pub liquidctl: BinaryProbe,
    pub openrgb: OpenRgbProbe,
    pub gaps: Vec<Gap>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HwmonChip {
    pub name: String,
    pub path: String,
    pub temps: Vec<LabeledValue>,
    pub fans: Vec<LabeledValue>,
    pub pwms: Vec<PwmNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabeledValue {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PwmNode {
    pub name: String,
    pub value: String,
    pub enable_exists: bool,
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DevNode {
    pub path: String,
    pub readable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinaryProbe {
    pub available: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRgbProbe {
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Gap {
    pub id: String,
    pub detail: String,
}
