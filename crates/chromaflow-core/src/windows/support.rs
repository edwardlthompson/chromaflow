//! Windows Support plan + apply (no bash/polkit).

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::support_progress::ProgressClock;
use crate::windows::lpc::{self, IoKind};
use crate::windows::lpc_client;

const APPLY_STEPS: &[(&str, &str, u32)] = &[
    ("lpc_lab", "Clear lab fixture", 2),
    ("pawnio", "Install PawnIO", 45),
    ("runtime_ps1", "Install Program Files helpers", 40),
    ("chromaflowd_bin", "Check PWM watchdog binary", 2),
    ("superio_lpc", "Start LPC SuperIO bridge", 8),
    ("superio_probe", "Probe motherboard chips", 10),
    ("chromaflowd", "Enable PWM watchdog", 8),
    ("gui_autostart", "Register GUI at logon", 3),
    ("openrgb_autostart", "Clear OpenRGB autostart", 4),
    ("openrgb_standalone", "Remove standalone OpenRGB", 20),
    ("steelseries_lock", "Release SteelSeries USB locks", 5),
    ("openrgb_engine", "Ensure private lighting engine", 25),
    ("openrgb_sdk", "Start lighting SDK server", 30),
    ("openrgb_zones", "Configure LED zones", 12),
    ("conflicts", "Scan competing controllers", 4),
];

fn lpc_label() -> &'static str {
    match lpc::kind() {
        IoKind::Mock => "lab_fixture",
        IoKind::Helper => {
            if lpc::bridge_live() {
                "bridge_running"
            } else {
                "bridge_missing"
            }
        }
        IoKind::Unavailable => "unavailable",
    }
}

fn find_runtime_ps1() -> Option<PathBuf> {
    if let Ok(r) = std::env::var("CHROMAFLOW_REPO") {
        let p = PathBuf::from(r).join("scripts").join("install-windows-runtime.ps1");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors().take(8) {
            let p = anc.join("scripts").join("install-windows-runtime.ps1");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn install_pawnio_winget() -> (bool, String) {
    if lpc::pawnio_installed() {
        return (true, "PawnIO already present under Program Files\\PawnIO.".into());
    }
    let mut cmd = Command::new("winget");
    cmd.args([
        "install",
        "-e",
        "--id",
        "namazso.PawnIO",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
    ]);
    let mut child = match crate::win_cmd::hidden(cmd).spawn() {
        Ok(c) => c,
        Err(e) => {
            return (
                false,
                format!("winget unavailable ({e}) — install PawnIO as Administrator, then reboot if needed."),
            );
        }
    };
    let deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(250));
            }
            _ => {
                let _ = child.kill();
                return (
                    false,
                    "winget install namazso.PawnIO timed out (120s) — run elevated scripts\\install-windows-runtime.ps1."
                        .into(),
                );
            }
        }
    };
    let ok = lpc::pawnio_installed();
    if ok {
        (
            true,
            format!("winget install namazso.PawnIO exit {status}; PawnIO files present."),
        )
    } else {
        (
            false,
            format!(
                "winget exit {status}; PawnIO still missing — reboot may be required after driver install (ChromaFlow never auto-reboots)."
            ),
        )
    }
}

fn run_elevated_runtime_ps1() -> (bool, String) {
    let Some(ps1) = find_runtime_ps1() else {
        return (
            false,
            "install-windows-runtime.ps1 not found (set CHROMAFLOW_REPO or run from a repo checkout)."
                .into(),
        );
    };
    let path = ps1.display().to_string().replace('\'', "''");
    let script = format!(
        "$p = Start-Process -FilePath powershell -Verb RunAs -Wait -PassThru \
         -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{path}'); \
         if ($null -eq $p) {{ 'uac_cancelled' }} else {{ \"exit:$($p.ExitCode)\" }}"
    );
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &script,
    ]);
    match crate::win_cmd::hidden_capture(cmd).output() {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let ok = text.starts_with("exit:0")
                || (lpc::superio_bridge_ready() && lpc::chromaflowd_installed());
            (
                ok,
                if text.is_empty() {
                    format!("elevated runtime install finished (script {})", ps1.display())
                } else {
                    format!("elevated runtime install: {text}")
                },
            )
        }
        Err(e) => (false, format!("elevated runtime install failed: {e}")),
    }
}

fn ensure_gui_autostart() -> String {
    let gui = Path::new(r"C:\Program Files\ChromaFlow\chromaflow-gui.exe");
    if !gui.is_file() {
        return "chromaflow-gui.exe missing under Program Files — skip HKCU Run".into();
    }
    let val = format!("\"{}\"", gui.display());
    let mut cmd = Command::new("reg");
    cmd.args([
        "add",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
        "/v",
        "ChromaFlow",
        "/t",
        "REG_SZ",
        "/d",
        &val,
        "/f",
    ]);
    if crate::win_cmd::hidden(cmd)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        format!("HKCU Run\\ChromaFlow -> {}", gui.display())
    } else {
        "failed to write HKCU Run\\ChromaFlow".into()
    }
}

fn restart_lpc_helper() -> String {
    let _ = lpc::ensure_helper();
    if lpc::bridge_live() {
        "LPC pipe answering".into()
    } else {
        "LPC pipe still down after ChromaFlowLPC start".into()
    }
}

/// Quit Gigabyte/Fan Control lockers, bounce the helper, retry SuperIO probe.
fn probe_and_shadow() -> (bool, String) {
    if !lpc::bridge_live() {
        let _ = restart_lpc_helper();
    }
    if !lpc::bridge_live() {
        return (false, "skip probe — LPC pipe not live".into());
    }
    let seize = crate::windows::conflicts::kill_running_competitors();
    let suppress = crate::windows::conflicts::suppress_gigabyte_fan_autostart();
    let _ = restart_lpc_helper();
    std::thread::sleep(Duration::from_millis(800));

    let root = lpc::live_root();
    let mut last = String::new();
    for attempt in 1..=3u8 {
        match lpc_client::probe() {
            Ok(chips) if chips.is_empty() => {
                last = format!(
                    "attempt {attempt}: SuperIO still empty (0xFFFF). {seize}; {suppress}"
                );
                std::thread::sleep(Duration::from_millis(500));
            }
            Ok(chips) => {
                let n = chips.len();
                let _ = lpc_client::materialize_from_probe(&root, &chips);
                if let Ok(sensors) = lpc_client::sensors() {
                    let _ = lpc_client::write_shadow(&root, &sensors);
                }
                return (
                    true,
                    format!(
                        "probed {n} SuperIO chip(s) into lpc-live (after: {seize}; {suppress})"
                    ),
                );
            }
            Err(e) => {
                last = format!("attempt {attempt}: {e}");
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
    // Bridge up + empty chips is not an Install-all package failure — Cooling shows gray headers.
    (
        true,
        format!(
            "LPC live; no ITE chips yet ({last}). EasyTune/Fan Control autostart suppressed — Rescan Cooling; motherboard PWM needs SuperIO IDs (not a missing package)."
        ),
    )
}

pub fn dry_run(_repo_root: &Path, _advanced: bool) -> Result<Value, String> {
    let pawnio = lpc::pawnio_installed();
    let daemon = lpc::chromaflowd_installed();
    let bin = lpc::superio_bridge_ready();
    let live = lpc::bridge_live();
    let lab = matches!(lpc::kind(), IoKind::Mock);
    let mode = lpc_label();

    let mut actions: Vec<String> = vec![
        "Install all: winget namazso.PawnIO (staged) + elevated install-windows-runtime.ps1 for chromaflowd/LPC/engine".into(),
        "Clear standalone OpenRGB Run/Startup — ChromaFlow ships a private headless lighting engine".into(),
        "Close Fan Control / Armoury / iCUE if they hold SuperIO or USB".into(),
    ];
    if !live {
        actions.insert(
            0,
            if bin {
                "BLOCKER: chromaflow-lpc is installed but the named pipe is down — Support → Install all starts ChromaFlowLPC.".into()
            } else {
                "BLOCKER: Motherboard fan PWM needs chromaflow-lpc — Support → Install all (ADR-0022). USB lighting still works.".into()
            },
        );
    }
    if lab {
        actions.insert(
            0,
            "Lab LPC fixture is ON — Auto-calibrate writes files only. Install all clears the lab marker.".into(),
        );
    }

    Ok(json!({
        "ok": true,
        "os": "windows",
        "apply": false,
        "source": "windows-support",
        "items": [
            {
                "id": "pawnio",
                "present": pawnio,
                "detail": if pawnio {
                    "PawnIO driver files found under Program Files\\PawnIO (kernel IO provider)."
                } else {
                    "PawnIO missing — Install all runs winget namazso.PawnIO (admin; reboot only if the driver will not start)."
                },
            },
            {
                "id": "chromaflowd",
                "present": daemon,
                "detail": if daemon {
                    "chromaflowd.exe present under Program Files\\ChromaFlow (PWM watchdog)."
                } else {
                    "chromaflowd missing — Install all copies release binaries via install-windows-runtime.ps1."
                },
            },
            {
                "id": "superio_lpc",
                "present": live,
                "detail": if live {
                    format!("LPC bridge running (mode={mode}, pipe ping ok).")
                } else if bin {
                    "BLOCKER: chromaflow-lpc.exe is installed but not answering on \\\\.\\pipe\\chromaflow-lpc. Install all starts ChromaFlowLPC.".to_string()
                } else {
                    "BLOCKER: chromaflow-lpc (elevated SuperIO↔PawnIO bridge) is not installed. Use Support → Install all.".to_string()
                },
            },
            {
                "id": "lpc_lab",
                "present": !lab,
                "detail": if lab {
                    "Lab fixture ACTIVE — Cooling cards are fake hwmon files, not chassis fans."
                } else {
                    "Lab fixture off (good for real hardware)."
                },
            },
            {
                "id": "webview2",
                "present": true,
                "detail": "WebView2 is required for chromaflow-gui (Tauri 2).",
            },
            {
                "id": "openrgb_engine",
                "present": crate::openrgb_spawn::resolve_bin().is_some(),
                "detail": "Private headless lighting engine under Program Files\\ChromaFlow\\engine\\OpenRGB (no tray).",
            },
            {
                "id": "hid_access",
                "present": true,
                "detail": "USB HID RGB uses hidapi. Exclusive locks from iCUE/Armoury may hide devices.",
            },
        ],
        "actions": actions,
    }))
}

/// Lighting + watchdog helpers. Never pretends lab PWM is real motherboard control.
pub fn apply(advanced: bool, extra: Option<&str>) -> Result<Value, String> {
    apply_with_progress(advanced, extra, None)
}

pub fn apply_with_progress(
    _advanced: bool,
    _extra: Option<&str>,
    on_progress: Option<Box<dyn FnMut(&crate::support_progress::SupportProgress) + Send>>,
) -> Result<Value, String> {
    let catalog: Vec<(&str, u32)> = APPLY_STEPS.iter().map(|(id, _, w)| (*id, *w)).collect();
    let mut clock = ProgressClock::new(&catalog, on_progress);
    let mut steps = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    let step = |clock: &mut ProgressClock,
                    steps: &mut Vec<Value>,
                    id: &str,
                    label: &str,
                    ok: bool,
                    detail: String| {
        clock.begin(id, label);
        clock.finish(id, label, &detail, ok);
        steps.push(json!({ "id": id, "ok": ok, "detail": detail, "label": label }));
    };

    clock.begin("lpc_lab", "Clear lab fixture");
    lpc::disable_lab_fixture();
    let detail = "Lab LPC fixture marker cleared (no silent file-only fans).".to_string();
    clock.finish("lpc_lab", "Clear lab fixture", &detail, true);
    steps.push(json!({"id": "lpc_lab", "ok": true, "detail": detail, "label": "Clear lab fixture"}));

    clock.begin("pawnio", "Install PawnIO");
    let (pawn_ok, pawn_detail) = install_pawnio_winget();
    clock.finish("pawnio", "Install PawnIO", &pawn_detail, pawn_ok);
    steps.push(json!({"id": "pawnio", "ok": pawn_ok, "detail": pawn_detail, "label": "Install PawnIO"}));
    if !pawn_ok {
        errors.push("PawnIO is not installed (or needs a user-approved reboot).".into());
    }

    let need_bins = !lpc::chromaflowd_installed() || !lpc::superio_bridge_ready();
    clock.begin("runtime_ps1", "Install Program Files helpers");
    let (rt_ok, rt_detail) = if need_bins || !lpc::bridge_live() {
        run_elevated_runtime_ps1()
    } else {
        (
            true,
            "chromaflowd + chromaflow-lpc already under Program Files; skipped elevated copy.".into(),
        )
    };
    clock.finish("runtime_ps1", "Install Program Files helpers", &rt_detail, rt_ok);
    steps.push(json!({"id": "runtime_ps1", "ok": rt_ok, "detail": rt_detail, "label": "Install Program Files helpers"}));
    if !rt_ok && need_bins {
        errors.push("Program Files layout incomplete — elevated install-windows-runtime.ps1 did not finish.".into());
    }

    let daemon = lpc::chromaflowd_installed();
    let d_detail = if daemon {
        "chromaflowd.exe found under Program Files\\ChromaFlow.".to_string()
    } else {
        "chromaflowd.exe still missing after Install all.".to_string()
    };
    step(
        &mut clock,
        &mut steps,
        "chromaflowd_bin",
        "Check PWM watchdog binary",
        daemon,
        d_detail,
    );
    if !daemon {
        errors.push("chromaflowd is not installed under Program Files\\ChromaFlow.".into());
    }

    clock.begin("superio_lpc", "Start LPC SuperIO bridge");
    let _ = lpc::ensure_helper();
    let bin = lpc::superio_bridge_ready();
    let live = lpc::bridge_live();
    let lpc_detail = if live {
        "LPC SuperIO bridge is running (pipe ping ok).".to_string()
    } else if bin {
        "BLOCKER: chromaflow-lpc.exe present but pipe down — start task ChromaFlowLPC or re-run Install all elevated.".to_string()
    } else {
        "BLOCKER: chromaflow-lpc not installed after Install all.".to_string()
    };
    clock.finish("superio_lpc", "Start LPC SuperIO bridge", &lpc_detail, live);
    steps.push(json!({"id": "superio_lpc", "ok": live, "detail": lpc_detail, "label": "Start LPC SuperIO bridge"}));
    if !live {
        errors.push("Motherboard PWM blocked: chromaflow-lpc named pipe is not answering.".into());
    }

    clock.begin("superio_probe", "Probe motherboard chips");
    let (probe_ok, probe_detail) = probe_and_shadow();
    // Empty SuperIO is hardware/BIOS lock — not a missing installer package.
    let probe_step_ok = probe_ok || live;
    clock.finish("superio_probe", "Probe motherboard chips", &probe_detail, probe_step_ok);
    steps.push(json!({"id": "superio_probe", "ok": probe_step_ok, "detail": probe_detail, "label": "Probe motherboard chips"}));

    clock.begin("chromaflowd", "Enable PWM watchdog");
    let (d_ok, d_msg) = match crate::pwm_daemon::enable() {
        Ok(msg) => (true, msg),
        Err(err) => {
            errors.push(format!("chromaflowd enable failed: {err}"));
            (false, err)
        }
    };
    clock.finish("chromaflowd", "Enable PWM watchdog", &d_msg, d_ok);
    steps.push(json!({"id": "chromaflowd", "ok": d_ok, "detail": d_msg, "label": "Enable PWM watchdog"}));

    let gui_detail = ensure_gui_autostart();
    step(&mut clock, &mut steps, "gui_autostart", "Register GUI at logon", true, gui_detail);

    let orgb_as = crate::openrgb_spawn::clear_openrgb_autostart();
    step(&mut clock, &mut steps, "openrgb_autostart", "Clear OpenRGB autostart", true, orgb_as);

    let orgb_rm = crate::windows::conflicts::uninstall_standalone_openrgb();
    step(&mut clock, &mut steps, "openrgb_standalone", "Remove standalone OpenRGB", true, orgb_rm);

    let ss = crate::windows::hid::release_steelseries_locks();
    step(&mut clock, &mut steps, "steelseries_lock", "Release SteelSeries USB locks", true, ss);

    clock.begin("openrgb_engine", "Ensure private lighting engine");
    let (eng_ok, eng_msg) = match crate::openrgb_engine::ensure_private_engine() {
        Ok(msg) => (true, msg),
        Err(err) => {
            errors.push(format!("OpenRGB engine: {err}"));
            (false, err)
        }
    };
    clock.finish("openrgb_engine", "Ensure private lighting engine", &eng_msg, eng_ok);
    steps.push(json!({"id": "openrgb_engine", "ok": eng_ok, "detail": eng_msg, "label": "Ensure private lighting engine"}));

    clock.begin("openrgb_sdk", "Start lighting SDK server");
    let _ = crate::openrgb_engine::configure_engine_ui();
    let (sdk_ok, sdk_msg) = match crate::openrgb_spawn::ensure_local_server() {
        Ok(msg) => (true, msg),
        Err(err) => {
            errors.push(format!("OpenRGB SDK: {err}"));
            (false, err)
        }
    };
    clock.finish("openrgb_sdk", "Start lighting SDK server", &sdk_msg, sdk_ok);
    steps.push(json!({"id": "openrgb_sdk", "ok": sdk_ok, "detail": sdk_msg, "label": "Start lighting SDK server"}));

    clock.begin("openrgb_zones", "Configure LED zones");
    let (z_ok, z_msg) = match crate::openrgb_zones::auto_configure_all() {
        Ok(msg) => {
            let remaining = crate::openrgb_zones::zero_led_linear_zones();
            if remaining > 0 {
                let _ = crate::openrgb_zones::auto_configure_all();
                let still = crate::openrgb_zones::zero_led_linear_zones();
                if still > 0 {
                    (
                        false,
                        format!("{msg}; {still} zone(s) still need a size — zone dialog may appear"),
                    )
                } else {
                    (true, format!("{msg} (retry ok)"))
                }
            } else {
                (true, msg)
            }
        }
        Err(err) => (false, err),
    };
    clock.finish("openrgb_zones", "Configure LED zones", &z_msg, z_ok);
    steps.push(json!({"id": "openrgb_zones", "ok": z_ok, "detail": z_msg, "label": "Configure LED zones"}));

    let conflicts = crate::conflicts::detect();
    let (c_ok, c_detail) = if conflicts.is_empty() {
        (true, "No competing fan/RGB controllers detected.".to_string())
    } else {
        let detail = format!("Uninstall before PWM: {}", conflicts.join(", "));
        errors.push(format!("Competing controllers: {}", conflicts.join(", ")));
        (false, detail)
    };
    step(&mut clock, &mut steps, "conflicts", "Scan competing controllers", c_ok, c_detail);

    let ok = errors.is_empty();
    Ok(json!({
        "ok": ok,
        "os": "windows",
        "apply": true,
        "steps": steps,
        "errors": errors,
        "catalog": APPLY_STEPS.iter().map(|(id, label, _)| json!({"id": id, "label": label})).collect::<Vec<_>>(),
        "hint": if ok {
            if probe_ok {
                "Windows Support apply finished — LPC live and SuperIO probe wrote lpc-live."
            } else {
                "Windows Support apply finished — LPC bridge answering; Rescan Cooling (chips may still be empty if SuperIO ports read 0xFF)."
            }
        } else {
            "Support apply finished with blockers — motherboard fans need a live chromaflow-lpc pipe (not just PawnIO)."
        },
    }))
}
