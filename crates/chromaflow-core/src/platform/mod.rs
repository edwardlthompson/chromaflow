//! OS backends for config paths, device discovery, and startup guards.
//! Feature code calls this module — never `cfg!(windows)` / `os.name` outside `platform/`.

#[cfg(not(windows))]
mod linux;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
use linux as os;
#[cfg(windows)]
use windows as os;

use crate::cooling::CoolingBackend;
use crate::types::Inventory;
use std::path::PathBuf;

pub fn cooling_backend() -> Box<dyn CoolingBackend> {
    os::cooling_backend()
}

pub fn config_dir() -> PathBuf {
    os::config_dir()
}

pub fn data_dir() -> PathBuf {
    os::data_dir()
}

pub fn runtime_dir() -> PathBuf {
    os::runtime_dir()
}

pub fn discover_inventory() -> Inventory {
    os::discover_inventory()
}

pub fn discover_cooling() -> Inventory {
    os::discover_cooling()
}

pub fn kernel_or_os_release() -> String {
    os::kernel_or_os_release()
}

pub fn euid() -> u32 {
    os::euid()
}

pub fn refuse_if_elevated() -> Result<(), String> {
    os::refuse_if_elevated()
}
