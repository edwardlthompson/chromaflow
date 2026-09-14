//! Watchdog loop plus lighting SDK keep-alive. --dry-run/--sdk never write PWM.

use chromaflow_core::collect_inventory;
use chromaflow_core::pwm_apply;
use chromaflow_core::pwm_curves;
use chromaflow_core::pwm_policy;
use std::thread;
use std::time::Duration;

fn once() -> bool {
    std::env::var("CHROMAFLOW_DAEMON_ONCE").ok().as_deref() == Some("1")
}

pub fn run_dry() -> i32 {
    eprintln!(
        "chromaflow daemon --dry-run: inventory watch only; no PWM; pwm*_enable failsafe={} (never 0)",
        pwm_policy::failsafe_enable()
    );
    loop {
        let inv = collect_inventory();
        eprintln!(
            "watch: hwmon={} conflicts={:?} openrgb={}",
            inv.hwmon.len(),
            inv.conflicts,
            inv.openrgb.status
        );
        if once() {
            return 0;
        }
        thread::sleep(Duration::from_secs(5));
    }
}

pub fn run_watch() -> i32 {
    eprintln!(
        "chromaflow daemon --watchdog: apply curves; failsafe pwm*_enable={} (never 0)",
        pwm_policy::failsafe_enable()
    );
    loop {
        let inv = collect_inventory();
        match pwm_curves::load().and_then(|file| pwm_apply::tick(&inv, &file)) {
            Ok(msg) => eprintln!("watchdog: {msg} conflicts={:?}", inv.conflicts),
            Err(err) => {
                eprintln!("watchdog: {err}; failsafe");
                let _ = pwm_apply::failsafe();
            }
        }
        if once() {
            return 0;
        }
        thread::sleep(Duration::from_secs(2));
    }
}

pub fn run_sdk() -> i32 {
    if !chromaflow_core::openrgb_spawn::sdk_opt_in() {
        eprintln!(
            "chromaflow daemon --sdk: OpenRGB spawn is off (native lighting). Set CHROMAFLOW_OPENRGB_SDK=1 to opt in; no PWM"
        );
        return 0;
    }
    eprintln!("chromaflow daemon --sdk: keep localhost lighting engine up; no PWM");
    chromaflow_core::openrgb_boot::run()
}

pub fn run_failsafe() -> i32 {
    match pwm_apply::failsafe() {
        Ok(n) => {
            eprintln!("chromaflow failsafe: restored {n} channel(s) to pwm*_enable=2");
            0
        }
        Err(err) => {
            eprintln!("chromaflow failsafe: {err}");
            1
        }
    }
}
