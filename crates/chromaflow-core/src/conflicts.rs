use std::path::Path;
use std::process::Command;

const NAMES: [&str; 3] = ["fancontrol", "coolercontrold", "fan2go"];

pub fn detect() -> Vec<String> {
    let mut found = Vec::new();
    for name in NAMES {
        if unit_exists(name) || process_exists(name) {
            found.push(name.to_string());
        }
    }
    found
}

fn unit_exists(name: &str) -> bool {
    let unit = format!("{name}.service");
    Path::new("/usr/lib/systemd/system").join(&unit).exists()
        || Path::new("/lib/systemd/system").join(&unit).exists()
        || Path::new("/etc/systemd/system").join(&unit).exists()
}

fn process_exists(name: &str) -> bool {
    Command::new("pidof")
        .arg(name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
