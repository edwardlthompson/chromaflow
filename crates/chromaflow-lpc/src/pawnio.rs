//! PawnIOLib.dll FFI (Windows). Loads LpcIO.bin / IsaBridgeEC.bin and runs ioctl_* calls.

#![cfg(windows)]

use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

/// Prefer HRESULT exports (`pawnio_*`). The `*_win32` variants return BOOL (0/1),
/// which we previously mis-checked as HRESULT and silently ignored failed port IO.
type OpenFn = unsafe extern "system" fn(*mut *mut c_void) -> i32;
type LoadFn = unsafe extern "system" fn(*mut c_void, *const u8, usize) -> i32;
type ExecFn = unsafe extern "system" fn(
    *mut c_void,
    *const c_char,
    *const u64,
    usize,
    *mut u64,
    usize,
    *mut usize,
) -> i32;
type CloseFn = unsafe extern "system" fn(*mut c_void) -> i32;

pub struct PawnIo {
    _lib: Library,
    handle: *mut c_void,
    exec: ExecFn,
    close: CloseFn,
}

unsafe impl Send for PawnIo {}

fn failed_hr(hr: i32) -> bool {
    hr < 0
}

impl PawnIo {
    pub fn open(dll: &Path, blob: &[u8]) -> Result<Self, String> {
        let lib = unsafe { Library::new(dll) }.map_err(|e| format!("PawnIOLib load: {e}"))?;
        unsafe {
            let open: Symbol<OpenFn> = lib
                .get(b"pawnio_open\0")
                .or_else(|_| lib.get(b"pawnio_open_win32\0"))
                .map_err(|e| format!("pawnio_open: {e}"))?;
            let load: Symbol<LoadFn> = lib
                .get(b"pawnio_load\0")
                .or_else(|_| lib.get(b"pawnio_load_win32\0"))
                .map_err(|e| format!("pawnio_load: {e}"))?;
            let exec: Symbol<ExecFn> = lib
                .get(b"pawnio_execute\0")
                .or_else(|_| lib.get(b"pawnio_execute_win32\0"))
                .map_err(|e| format!("pawnio_execute: {e}"))?;
            let close: Symbol<CloseFn> = lib
                .get(b"pawnio_close\0")
                .or_else(|_| lib.get(b"pawnio_close_win32\0"))
                .map_err(|e| format!("pawnio_close: {e}"))?;
            let open_fn = *open;
            let load_fn = *load;
            let exec_fn = *exec;
            let close_fn = *close;
            let mut handle: *mut c_void = ptr::null_mut();
            let hr = open_fn(&mut handle);
            if failed_hr(hr) || handle.is_null() {
                return Err(format!("pawnio_open HRESULT 0x{hr:08X}"));
            }
            let hr = load_fn(handle, blob.as_ptr(), blob.len());
            if failed_hr(hr) {
                close_fn(handle);
                return Err(format!("pawnio_load HRESULT 0x{hr:08X}"));
            }
            Ok(Self {
                _lib: lib,
                handle,
                exec: exec_fn,
                close: close_fn,
            })
        }
    }

    pub fn execute(&self, name: &str, input: &[u64], out_len: usize) -> Result<Vec<u64>, String> {
        let cname = std::ffi::CString::new(name).map_err(|e| e.to_string())?;
        let mut out = if out_len == 0 {
            Vec::new()
        } else {
            vec![0u64; out_len]
        };
        let mut ret = 0usize;
        let hr = unsafe {
            (self.exec)(
                self.handle,
                cname.as_ptr(),
                if input.is_empty() {
                    ptr::null()
                } else {
                    input.as_ptr()
                },
                input.len(),
                if out_len == 0 {
                    ptr::null_mut()
                } else {
                    out.as_mut_ptr()
                },
                out_len,
                &mut ret,
            )
        };
        // HRESULT: S_OK == 0 success; negative = failure.
        if failed_hr(hr) {
            return Err(format!("{name} HRESULT 0x{hr:08X}"));
        }
        if out_len > 0 {
            out.truncate(ret.min(out_len));
        }
        Ok(out)
    }

    pub fn select_slot(&self, slot: u8) -> Result<(), String> {
        self.execute("ioctl_select_slot", &[u64::from(slot)], 0)?;
        Ok(())
    }

    pub fn find_bars(&self) -> Result<(), String> {
        self.execute("ioctl_find_bars", &[], 0)?;
        Ok(())
    }

    pub fn pio_inb(&self, port: u16) -> Result<u8, String> {
        let out = self.execute("ioctl_pio_inb", &[u64::from(port)], 1)?;
        Ok(out.first().copied().unwrap_or(0) as u8)
    }

    pub fn pio_outb(&self, port: u16, value: u8) -> Result<(), String> {
        self.execute("ioctl_pio_outb", &[u64::from(port), u64::from(value)], 0)?;
        Ok(())
    }

    pub fn sio_inb(&self, reg: u8) -> Result<u8, String> {
        let out = self.execute("ioctl_superio_inb", &[u64::from(reg)], 1)?;
        Ok(out.first().copied().unwrap_or(0) as u8)
    }

    pub fn sio_inw(&self, reg: u8) -> Result<u16, String> {
        let out = self.execute("ioctl_superio_inw", &[u64::from(reg)], 1)?;
        Ok(out.first().copied().unwrap_or(0) as u16)
    }

    pub fn sio_outb(&self, reg: u8, value: u8) -> Result<(), String> {
        self.execute(
            "ioctl_superio_outb",
            &[u64::from(reg), u64::from(value)],
            0,
        )?;
        Ok(())
    }
}

impl Drop for PawnIo {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                (self.close)(self.handle);
            }
            self.handle = ptr::null_mut();
        }
    }
}
