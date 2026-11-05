#![deny(unsafe_code)]

//! Inventory plus localhost lighting apply. PWM writes only via watchdog (ADR-0018).

pub mod allowlist;
pub mod arena_apply;
pub mod conflicts;
pub mod cooling;
pub mod cpu_load;
pub mod fs_mode;
pub mod fusion_hid;
pub mod gaps;
pub mod gauges;
pub mod gpu_apply;
pub mod hwmon;
pub mod keychron_apply;
pub mod keychron_preview;
pub mod lighting;
pub mod lighting_apply;
pub mod lighting_broadcast;
pub mod lighting_cycle;
pub mod lighting_port;
pub mod liquidctl_apply;
pub mod names;
pub mod nodes;
pub mod nvidia_fan_apply;
pub mod nvidia_fans;
pub mod nvidia_smi;
pub mod openrgb;
pub mod openrgb_apply;
pub mod openrgb_boot;
pub mod openrgb_dled;
pub mod openrgb_engine;
pub mod openrgb_mode;
pub mod openrgb_parse;
pub mod openrgb_preview;
pub mod openrgb_proto;
pub mod openrgb_sandbox;
pub mod openrgb_spawn;
pub mod openrgb_zones;
pub mod prime_apply;
pub mod paths;
pub mod platform;
pub mod privilege;
pub mod probes;
pub mod profiles;
pub mod pwm_apply;
pub mod pwm_calibrate;
pub mod pwm_curves;
pub mod pwm_daemon;
pub mod pwm_hyst;
pub mod pwm_policy;
pub mod pwm_recipe;
pub mod pwm_scheme;
pub mod scan;
pub mod support;
pub mod support_progress;
pub mod types;
pub mod update_check;
pub mod update_fetch;
pub mod win_cmd;
#[cfg(windows)]
pub mod win_gauges;

#[cfg(windows)]
pub mod windows;

pub use privilege::refuse_if_root;
pub use scan::{collect_cooling, collect_inventory};
pub use types::Inventory;
