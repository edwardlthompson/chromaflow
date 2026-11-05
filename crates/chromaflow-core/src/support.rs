use serde::Serialize;
use serde_json::Value;
use std::path::Path;
#[cfg(not(windows))]
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
pub struct SupportPlan {
    pub ok: bool,
    pub source: String,
    pub plan: Value,
}

pub fn dry_run(repo_root: &Path, advanced: bool) -> Result<SupportPlan, String> {
    #[cfg(windows)]
    {
        let _ = advanced;
        let plan = crate::windows::support::dry_run(repo_root, advanced)?;
        return Ok(SupportPlan {
            ok: plan.get("ok").and_then(Value::as_bool).unwrap_or(false),
            source: "windows-support".into(),
            plan,
        });
    }
    #[cfg(not(windows))]
    {
        let script = repo_root.join("scripts/install-support.sh");
        if !script.is_file() {
            return Err("scripts/install-support.sh missing".into());
        }
        let mut cmd = Command::new("bash");
        cmd.arg(script).arg("--dry-run").current_dir(repo_root);
        if advanced {
            cmd.arg("--advanced");
        }
        cmd.stdin(Stdio::null());
        let out = cmd.output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).into_owned());
        }
        let plan: Value =
            serde_json::from_slice(&out.stdout).map_err(|e| format!("support JSON: {e}"))?;
        Ok(SupportPlan {
            ok: plan.get("ok").and_then(Value::as_bool).unwrap_or(false),
            source: "install-support.sh --dry-run".into(),
            plan,
        })
    }
}

pub fn apply(advanced: bool, extra: Option<&str>) -> Result<Value, String> {
    apply_with_progress(advanced, extra, None)
}

pub fn apply_with_progress(
    advanced: bool,
    extra: Option<&str>,
    on_progress: Option<Box<dyn FnMut(&crate::support_progress::SupportProgress) + Send>>,
) -> Result<Value, String> {
    #[cfg(windows)]
    {
        return crate::windows::support::apply_with_progress(advanced, extra, on_progress);
    }
    #[cfg(not(windows))]
    {
        let _ = on_progress;
        let pinned = Path::new("/usr/libexec/chromaflow/install-support.sh");
        if !pinned.is_file() {
            return Err(
                "chromaflow-helper is not installed at /usr/libexec/chromaflow/install-support.sh"
                    .into(),
            );
        }
        if let Some(id) = extra {
            if !matches!(
                id,
                "it87-dkms"
                    | "liquidctl"
                    | "linux-modules-extra"
                    | "i2c-dev"
                    | "i2c-piix4"
                    | "it87"
                    | "nct6775"
                    | "k10temp"
                    | "jc42"
                    | "spd5118"
                    | "gigabyte_wmi"
                    | "udev"
                    | "group_i2c"
                    | "group_plugdev"
                    | "pwm_acl"
                    | "i2c-nct6775"
                    | "i2c-nvidia-gpu"
            ) {
                return Err("unknown extra".into());
            }
        }
        let mut cmd = Command::new("pkexec");
        cmd.arg(pinned).arg("--apply").stdin(Stdio::null());
        if let Some(id) = extra {
            cmd.arg("--only").arg(id);
        } else if advanced {
            cmd.arg("--advanced");
        }
        let out = cmd.output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        json_from_output(&stdout)
            .ok_or_else(|| brief(&format!("pkexec apply failed: {stderr}{stdout}")))
    }
}

#[allow(dead_code)]
fn json_from_output(stdout: &str) -> Option<Value> {
    let text = stdout.trim();
    if let Ok(v) = serde_json::from_str(text) {
        return Some(v);
    }
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str(&text[start..=end]).ok()
}

#[cfg_attr(windows, allow(dead_code))]
fn brief(msg: &str) -> String {
    let text = msg.trim();
    let count = text.chars().count();
    if count <= 180 {
        text.into()
    } else {
        format!("{}…", text.chars().take(180).collect::<String>())
    }
}

pub fn competitors_plan(repo_root: &Path) -> Result<Value, String> {
    #[cfg(windows)]
    {
        let _ = repo_root;
        let running = crate::conflicts::detect();
        let installed = crate::windows::conflicts::installed_names();
        let mut would = running.clone();
        for name in &installed {
            if !would.iter().any(|w| w == name) {
                would.push(name.clone());
            }
        }
        Ok(serde_json::json!({
            "ok": true,
            "os": "windows",
            "detected": would,
            "running": running,
            "installed": installed,
            "would_remove": would,
            "detail": "Take control quits running lockers and stops SteelSeries GG autostart. Uninstall leftovers from Settings → Apps if they remain.",
            "items": would.iter().map(|name| serde_json::json!({
                "id": name,
                "detail": format!("Quit or uninstall {name} so ChromaFlow can control fans and lighting.")
            })).collect::<Vec<_>>()
        }))
    }
    #[cfg(not(windows))]
    {
        let script = repo_root.join("scripts/manage-competitors.sh");
        if !script.is_file() {
            return Err("scripts/manage-competitors.sh missing".into());
        }
        let out = Command::new("bash")
            .arg(script)
            .arg("--dry-run")
            .current_dir(repo_root)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).into_owned());
        }
        serde_json::from_slice(&out.stdout).map_err(|e| format!("competitors JSON: {e}"))
    }
}

pub fn competitors_remove() -> Result<Value, String> {
    #[cfg(windows)]
    {
        Ok(crate::windows::conflicts::seize_control())
    }
    #[cfg(not(windows))]
    {
        let pinned = Path::new("/usr/libexec/chromaflow/manage-competitors.sh");
        if !pinned.is_file() {
            return Err(
                "chromaflow-helper is not installed at /usr/libexec/chromaflow/manage-competitors.sh"
                    .into(),
            );
        }
        let out = Command::new("pkexec")
            .arg(pinned)
            .arg("--remove-apply")
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        if let Ok(plan) = serde_json::from_str::<Value>(stdout.trim()) {
            return Ok(plan);
        }
        if !out.status.success() {
            return Err(format!("pkexec remove failed: {stderr}{stdout}"));
        }
        Err(format!("remove JSON: {stderr}{stdout}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{brief, json_from_output};

    #[test]
    fn json_ignores_pwm_acl_banner() {
        let raw = "chromaflow pwm-acl: plugdev 0660 on 20 pwm nodes\n{\"ok\":true,\"extras\":[{\"id\":\"pwm_acl\",\"present\":true}]}\n";
        let v = json_from_output(raw).expect("json");
        assert_eq!(v["extras"][0]["id"], "pwm_acl");
        assert_eq!(v["extras"][0]["present"], true);
    }

    #[test]
    fn brief_truncates() {
        let long = "e".repeat(200);
        assert!(brief(&long).ends_with('…'));
        assert!(brief(&long).chars().count() <= 181);
    }
}
