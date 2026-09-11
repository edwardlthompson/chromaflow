use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
pub struct SupportPlan {
    pub ok: bool,
    pub source: String,
    pub plan: Value,
}

pub fn dry_run(repo_root: &Path, advanced: bool) -> Result<SupportPlan, String> {
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
