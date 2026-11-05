//! Confirmed take-over: Balanced scheme, calibrate, watchdog. Never silent 0%.

use chromaflow_core::collect_inventory;
use chromaflow_core::pwm_apply;
use chromaflow_core::pwm_calibrate;
use chromaflow_core::pwm_curves;
use chromaflow_core::pwm_daemon;
use chromaflow_core::pwm_policy;
use chromaflow_core::pwm_scheme;
use std::fs;

fn skip_daemon() -> bool {
    std::env::var("CHROMAFLOW_SKIP_DAEMON").ok().as_deref() == Some("1")
}

pub fn run_takeover() -> i32 {
    let inv = collect_inventory();
    let mut file = pwm_scheme::balanced(&inv);
    if file.channels.is_empty() {
        eprintln!("cooling --takeover: no writable fans or pumps");
        return 1;
    }
    if let Err(err) = pwm_curves::save(&file) {
        eprintln!("{err}");
        return 1;
    }
    match pwm_apply::tick(&inv, &file) {
        Ok(msg) => eprintln!("takeover: {msg}"),
        Err(err) => {
            eprintln!("takeover: {err}");
            return 1;
        }
    }
    if !skip_daemon() {
        match pwm_daemon::enable() {
            Ok(msg) => eprintln!("{msg}"),
            Err(err) => eprintln!("chromaflowd: {err}"),
        }
    }
    let mut ok_n = 0u32;
    let mut fail_n = 0u32;
    let mut silent_n = 0u32;
    let total = file
        .channels
        .iter()
        .filter(|ch| ch.chip != "nvidia" && ch.enabled && pwm_policy::valid_dir(&ch.dir))
        .count();
    for ch in file.channels.clone() {
        if ch.chip == "nvidia" || !ch.enabled || !pwm_policy::valid_dir(&ch.dir) {
            continue;
        }
        // Sweep every writable header (incl. 0 RPM) so no-tach is learned; never abort the board.
        match pwm_calibrate::sweep(&inv, &ch.dir, &ch.pwm, file.allow_zero) {
            Ok(rows) => {
                let key = format!("{}/{}", ch.dir, ch.pwm);
                let empty = rows.last().map(|r| r[1]).unwrap_or(0) == 0;
                file.calibration
                    .insert(key, if empty { Vec::new() } else { rows.clone() });
                if empty {
                    silent_n += 1;
                    eprintln!("calibrate {}/{}: no tach (0 RPM @ 100%)", ch.dir, ch.pwm);
                } else {
                    ok_n += 1;
                    eprintln!("calibrate {}/{} {:?}", ch.dir, ch.pwm, rows.last());
                }
            }
            Err(err) => {
                fail_n += 1;
                eprintln!("calibrate {}/{}: {err}", ch.dir, ch.pwm);
            }
        }
    }
    eprintln!(
        "calibrate board: {total} headers, {ok_n} with tach, {silent_n} no-tach, {fail_n} failed"
    );
    let _ = pwm_curves::save(&file);
    match pwm_apply::tick(&inv, &file) {
        Ok(msg) => {
            eprintln!(
                "scheme balanced: {msg}; failsafe enable={}",
                pwm_policy::failsafe_enable()
            );
            let _ = fs::read_to_string(pwm_apply::owned_path()).map(|t| eprintln!("owned:\n{t}"));
            0
        }
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}
