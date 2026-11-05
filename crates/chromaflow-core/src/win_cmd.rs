//! Windows process launches must never flash a console or steal focus.

use std::process::{Command, Output, Stdio};

/// Apply CREATE_NO_WINDOW + null stdio so child tools never pop a console.
#[cfg(windows)]
pub fn hidden(mut cmd: Command) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

/// Like [`hidden`], but keep stdout/stderr for callers that parse `.output()`.
#[cfg(windows)]
pub fn hidden_capture(mut cmd: Command) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW).stdin(Stdio::null());
    cmd
}

#[cfg(not(windows))]
pub fn hidden(cmd: Command) -> Command {
    cmd
}

#[cfg(not(windows))]
pub fn hidden_capture(cmd: Command) -> Command {
    cmd
}

/// Run `program` with args. On Linux wraps with `timeout <secs>`; on Windows never
/// invokes `timeout.exe` (that binary waits N seconds and steals focus).
pub fn run_capped(secs: &str, program: &str, args: &[&str]) -> std::io::Result<Output> {
    #[cfg(windows)]
    {
        let _ = secs;
        let mut cmd = Command::new(program);
        cmd.args(args);
        hidden_capture(cmd).output()
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("timeout");
        cmd.arg(secs).arg(program).args(args);
        cmd.output()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn helpers_compile() {
        let _ = super::hidden(std::process::Command::new("true"));
        let _ = super::hidden_capture(std::process::Command::new("true"));
    }
}
