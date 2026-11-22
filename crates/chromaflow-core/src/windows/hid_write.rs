//! Write HID reports on Windows via a tiny compiled helper (no unsafe in this crate).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn helper_exe() -> Result<PathBuf, String> {
    let dir = crate::paths::data_dir().join("bin");
    let exe = dir.join("CfHidW.exe");
    let cs = dir.join("CfHidW.cs");
    match helper_source() {
        Ok(src) => {
            let src_changed = fs::read_to_string(&cs).ok().as_deref() != Some(src.as_str());
            if exe.is_file() && !src_changed {
                return Ok(exe);
            }
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            fs::write(&cs, &src).map_err(|e| e.to_string())?;
            let csc = r"C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe";
            if !std::path::Path::new(csc).is_file() {
                if exe.is_file() {
                    return Ok(exe);
                }
                return Err("csc.exe not found to build HID helper".into());
            }
            let _ = fs::remove_file(&exe);
            let mut cmd = Command::new(csc);
            cmd.args([
                "/nologo",
                "/optimize+",
                &format!("/out:{}", exe.display()),
                &cs.display().to_string(),
            ]);
            let out = crate::win_cmd::hidden_capture(cmd)
                .output()
                .map_err(|e| e.to_string())?;
            if !out.status.success() || !exe.is_file() {
                return Err(format!(
                    "compile CfHidW failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            Ok(exe)
        }
        Err(e) => {
            // Keep lighting working when repo/install source is missing but a prior helper remains.
            if exe.is_file() {
                return Ok(exe);
            }
            Err(e)
        }
    }
}

fn helper_source() -> Result<String, String> {
    let data_cs = crate::paths::data_dir().join("bin").join("CfHidW.cs");
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/windows/CfHidW.cs"),
        PathBuf::from(r"C:\Program Files\ChromaFlow\scripts\windows\CfHidW.cs"),
        data_cs,
    ];
    for p in candidates {
        if let Ok(s) = fs::read_to_string(&p) {
            if !s.trim().is_empty() {
                return Ok(s);
            }
        }
    }
    Err("CfHidW.cs missing".into())
}

pub fn write_output_report(vid: &str, pid: &str, report: &[u8]) -> Result<String, String> {
    let exe = helper_exe()?;
    let hex: String = report.iter().map(|b| format!("{b:02X}")).collect();
    let mut cmd = Command::new(&exe);
    cmd.args([vid, pid, &hex]);
    let out = crate::win_cmd::hidden_capture(cmd)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !out.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(if stdout.is_empty() {
        format!("HID {vid}:{pid} wrote {} bytes", report.len())
    } else {
        stdout
    })
}

pub fn arena_color(rgb: [u8; 3]) -> Result<String, String> {
    let report = crate::arena_apply::color_report(rgb);
    write_output_report("1038", "1a00", &report).map(|m| format!("Arena 7: {m}"))
}

pub fn prime_color(rgb: [u8; 3]) -> Result<String, String> {
    let report = crate::prime_apply::color_report(rgb);
    let live = write_output_report("1038", "1856", &report)?;
    let save = crate::prime_apply::save_report();
    let _ = write_output_report("1038", "1856", &save);
    Ok(format!("Prime Neo: {live}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn ships_cfhidw_source() {
        let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/windows/CfHidW.cs");
        let src = std::fs::read_to_string(&p).expect("scripts/windows/CfHidW.cs must ship");
        assert!(src.contains("HidD_SetOutputReport"));
        assert!(src.contains("CfHidW"));
    }
}
