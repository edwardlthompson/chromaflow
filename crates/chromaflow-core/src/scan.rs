//! Inventory entry points — OS details live in `platform/`.

use crate::platform;
use crate::types::Inventory;

pub fn collect_inventory() -> Inventory {
    platform::discover_inventory()
}

pub fn collect_cooling() -> Inventory {
    platform::discover_cooling()
}
