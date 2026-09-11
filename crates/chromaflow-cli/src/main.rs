use chromaflow_core::support;
use chromaflow_core::{collect_inventory, refuse_if_root};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn repo_root() -> PathBuf {
    env::var("CHROMAFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn print_help() {
    eprintln!(
        "chromaflow — unprivileged inventory CLI\n\
         Usage: chromaflow <sensors|devices|rescan|support> [--dry-run] [--advanced]\n\
         Do not run as root. This binary never writes PWM."
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
    if rest.iter().any(|a| a == "--apply" || a == "set-pwm") {
        eprintln!("refusing apply/PWM flags; inventory is read-only");
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
        _ => {
            print_help();
            ExitCode::from(2)
        }
    }
}
