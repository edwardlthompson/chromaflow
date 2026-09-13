use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {
    pub hwmon: Vec<HwmonChip>,
    pub gpu_fans: Vec<GpuFan>,
    pub hidraw: Vec<DevNode>,
    pub hid_rgb: Vec<HidRgb>,
    pub i2c: Vec<DevNode>,
    pub liquidctl: BinaryProbe,
    pub liquidctl_devices: Vec<String>,
    pub openrgb: OpenRgbProbe,
    pub kernel_release: String,
    pub gaps: Vec<Gap>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HidRgb {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub vendor_id: String,
    pub product_id: String,
    pub readable: bool,
    pub hid_name: String,
    pub manufacturer: String,
    pub product: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuFan {
    pub id: String,
    pub label: String,
    pub percent: Option<u8>,
    pub rpm: Option<u32>,
    pub writable: bool,
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
    pub controllers: Vec<RgbDevice>,
    pub sandboxed: bool,
    pub engine_missing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RgbDevice {
    pub name: String,
    pub protocol: String,
    pub leds: u16,
    pub color: String,
    pub modes: Vec<String>,
    pub active_mode: i32,
    pub led_names: Vec<String>,
    pub grid_w: u16,
    pub grid_h: u16,
    pub grid: Vec<i32>,
    pub led_colors: Vec<String>,
}

impl RgbDevice {
    pub fn sdk(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            protocol: "OpenRGB SDK".into(),
            leds: 0,
            color: String::new(),
            modes: Vec::new(),
            active_mode: 0,
            led_names: Vec::new(),
            grid_w: 0,
            grid_h: 0,
            grid: Vec::new(),
            led_colors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Gap {
    pub id: String,
    pub detail: String,
}
