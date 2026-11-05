//! Windows-only cooling / lighting support. No unsigned WinRing0.

pub mod conflicts;
pub mod hid;
pub mod hid_write;
pub mod lpc;
pub mod lpc_client;
pub mod superio;
pub mod support;
