use chromaflow_core::lighting_apply;
use chromaflow_core::lighting_broadcast;
use chromaflow_core::lighting_cycle;
use chromaflow_core::profiles;
use chromaflow_core::support;
use chromaflow_core::{collect_inventory, refuse_if_root};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

mod cooling;
mod daemon;

fn repo_root() -> PathBuf {
    env::var("CHROMAFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn print_help() {
    eprintln!(
        "chromaflow — unprivileged inventory CLI\n\
         Usage: chromaflow <sensors|devices|rescan|support|profiles|daemon|cooling|rgb> [...]\n\
         support: [--advanced] [--apply] [--only NAME]\n\
         rgb: --backend openrgb|liquidctl|arena|prime|keychron|msi_gpu --device NAME --color RRGGBB [--mode NAME] [--led N] [--broadcast] [--cycle] [--seconds N] [--poll]\n\
         daemon: --dry-run | --watchdog | --failsafe | --sdk\n\
         cooling: --takeover\n\
         Do not run as root. PWM duty only via --watchdog or cooling --takeover (never silent 0%)."
    );
}

fn main() -> ExitCode {
    if let Err(err) = refuse_if_root() {
        eprintln!("{err}");
        return ExitCode::from(1);
    }
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_help();
        return ExitCode::from(2);
    };
    if cmd == "-h" || cmd == "--help" {
        print_help();
        return ExitCode::SUCCESS;
    }
    let rest: Vec<String> = args.collect();
    if rest.iter().any(|a| a == "set-pwm") {
        eprintln!("refusing PWM flags; this binary never writes PWM");
        return ExitCode::from(2);
    }
    match cmd.as_str() {
        "sensors" | "devices" | "rescan" => {
            let inv = collect_inventory();
            println!("{}", serde_json::to_string_pretty(&inv).expect("json"));
            ExitCode::SUCCESS
        }
        "support" => {
            let advanced = rest.iter().any(|a| a == "--advanced");
            let extra = rest.windows(2).find(|w| w[0] == "--only").map(|w| w[1].as_str());
            if rest.iter().any(|a| a == "--apply") {
                match support::apply(advanced, extra) {
                    Ok(plan) => {
                        println!("{}", serde_json::to_string_pretty(&plan).expect("json"));
                        if plan.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from(1)
                        }
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        ExitCode::from(1)
                    }
                }
            } else {
                match support::dry_run(&repo_root(), advanced) {
                    Ok(plan) => {
                        println!("{}", serde_json::to_string_pretty(&plan).expect("json"));
                        if plan.ok {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from(1)
                        }
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        ExitCode::from(1)
                    }
                }
            }
        }
        "profiles" => match profiles::load(&profiles::config_dir()) {
            Ok(file) => {
                println!("{}", serde_json::to_string_pretty(&file).expect("json"));
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::from(1)
            }
        },
        "daemon" => {
            if rest.iter().any(|a| a == "--dry-run") {
                ExitCode::from(daemon::run_dry() as u8)
            } else if rest.iter().any(|a| a == "--watchdog") {
                ExitCode::from(daemon::run_watch() as u8)
            } else if rest.iter().any(|a| a == "--failsafe") {
                ExitCode::from(daemon::run_failsafe() as u8)
            } else if rest.iter().any(|a| a == "--sdk") {
                ExitCode::from(daemon::run_sdk() as u8)
            } else {
                eprintln!("daemon requires --dry-run, --watchdog, --failsafe, or --sdk");
                ExitCode::from(2)
            }
        }
        "cooling" => {
            if rest.iter().any(|a| a == "--takeover") {
                ExitCode::from(cooling::run_takeover() as u8)
            } else {
                eprintln!("cooling requires --takeover");
                ExitCode::from(2)
            }
        }
        "rgb" => rgb_cmd(&rest),
        _ => {
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_keychron_poll() {
    let fx = chromaflow_core::keychron_apply::poll_effect();
    let hue = chromaflow_core::keychron_apply::poll_hue();
    println!(
        "keychron effect={} hue={}",
        fx.map(|n| n.to_string()).unwrap_or_else(|| "none".into()),
        hue.map(|n| n.to_string()).unwrap_or_else(|| "none".into()),
    );
}

fn rgb_cmd(rest: &[String]) -> ExitCode {
    let mut backend = "openrgb".to_string();
    let mut device = String::new();
    let mut color = String::new();
    let mut mode = String::new();
    let mut led: Option<u16> = None;
    let mut broadcast = false;
    let mut cycle = false;
    let mut poll = false;
    let mut seconds = 3u64;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--backend" => {
                backend = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--device" => {
                device = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--color" => {
                color = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--mode" => {
                mode = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--led" => {
                led = rest.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "--seconds" => {
                seconds = rest
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .filter(|n| *n > 0)
                    .unwrap_or(3);
                i += 2;
            }
            "--broadcast" => {
                broadcast = true;
                i += 1;
            }
            "--cycle" => {
                cycle = true;
                i += 1;
            }
            "--poll" => {
                poll = true;
                i += 1;
            }
            flag => {
                eprintln!("unknown rgb flag {flag}");
                return ExitCode::from(2);
            }
        }
    }
    if poll {
        if color.is_empty() && !broadcast && !cycle {
            print_keychron_poll();
            return ExitCode::SUCCESS;
        }
        if !cycle {
            print_keychron_poll();
        }
    }
    if cycle {
        lighting_cycle::set_running(true, 128);
        let until = Instant::now() + Duration::from_secs(seconds.max(1));
        while Instant::now() < until {
            thread::sleep(Duration::from_millis(150));
            if poll {
                println!("cycle hue={}", lighting_cycle::current_hue());
            }
        }
        lighting_cycle::set_running(false, 128);
        if poll {
            print_keychron_poll();
        }
        return ExitCode::SUCCESS;
    }
    if broadcast {
        if color.is_empty() {
            eprintln!("rgb --broadcast requires --color RRGGBB");
            return ExitCode::from(2);
        }
        let mode = if mode.trim().is_empty() {
            "Solid Color"
        } else {
            mode.as_str()
        };
        return match lighting_broadcast::broadcast(&color, mode) {
            Ok(msg) => {
                println!("{msg}");
                if poll {
                    print_keychron_poll();
                }
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::from(1)
            }
        };
    }
    if device.is_empty() || color.is_empty() {
        eprintln!("rgb requires --device and --color RRGGBB");
        return ExitCode::from(2);
    }
    let result = if let Some(led) = led {
        lighting_apply::apply_led(&backend, &device, led, &color)
    } else if !mode.trim().is_empty() {
        lighting_apply::apply_mode(&backend, &device, &mode, &color)
    } else {
        lighting_apply::apply(&backend, &device, &color)
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
