//! Gigabyte / ISA bus synchronization mutant used by LHM, Fan Control, and OEM tools.
//! LpcIO.p warns: acquire before select_slot / find_bars / pio / superio ioctls.

#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::Duration;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{
    CreateMutexW, ReleaseMutex, WaitForSingleObject, INFINITE,
};

/// Usermode name used by LHM / Fan Control / Argus (maps to BaseNamedObjects).
const MUTANT: &str = r"Global\Access_ISABUS.HTP.Method";

pub struct IsaBusGuard {
    handle: HANDLE,
}

impl IsaBusGuard {
    /// Create-or-open the ISA bus mutant and wait up to `timeout`.
    pub fn acquire(timeout: Duration) -> Result<Self, String> {
        let wide: Vec<u16> = OsStr::new(MUTANT)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: CreateMutexW with null security; name is static UTF-16.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
        if handle.is_null() {
            return Err(format!(
                "CreateMutexW({MUTANT}): {}",
                std::io::Error::last_os_error()
            ));
        }
        let ms = timeout.as_millis().min(u32::MAX as u128) as u32;
        let wait = if ms == 0 { INFINITE } else { ms };
        let rc = unsafe { WaitForSingleObject(handle, wait) };
        if rc == WAIT_OBJECT_0 || rc == 0x0000_0080 {
            // WAIT_ABANDONED_0 == 0x80 — previous owner died; we still own it.
            Ok(Self { handle })
        } else if rc == WAIT_TIMEOUT {
            unsafe {
                let _ = CloseHandle(handle);
            }
            Err(format!(
                "ISA bus mutant busy ({MUTANT}) — quit EasyTune/GCC/Fan Control and retry"
            ))
        } else {
            let err = std::io::Error::last_os_error();
            unsafe {
                let _ = CloseHandle(handle);
            }
            Err(format!("WaitForSingleObject({MUTANT}): {err}"))
        }
    }
}

impl Drop for IsaBusGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                let _ = ReleaseMutex(self.handle);
                let _ = CloseHandle(self.handle);
            }
            self.handle = std::ptr::null_mut();
        }
    }
}
