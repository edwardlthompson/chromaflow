#![deny(unsafe_code)]

//! Read-only cooling/lighting inventory. PWM writes are forbidden (ADR-0010).

pub mod allowlist;
pub mod conflicts;
pub mod gaps;
pub mod hwmon;
pub mod names;
pub mod nodes;
pub mod privilege;
pub mod probes;
pub mod scan;
pub mod support;
pub mod types;

pub use privilege::refuse_if_root;
pub use scan::collect_inventory;
pub use types::Inventory;
