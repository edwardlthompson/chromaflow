use crate::types::{BinaryProbe, OpenRgbProbe};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::process::Command;
use std::time::Duration;

pub fn liquidctl() -> BinaryProbe {
    match Command::new("liquidctl").arg("--version").output() {
        Ok(out) if out.status.success() => BinaryProbe {
            available: true,
            detail: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        },
        Ok(out) => BinaryProbe {
            available: false,
            detail: format!("exit {}", out.status),
        },
        Err(err) => BinaryProbe {
            available: false,
            detail: err.to_string(),
        },
    }
}

pub fn openrgb() -> OpenRgbProbe {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6742);
    match TcpStream::connect_timeout(&addr, Duration::from_millis(200)) {
        Ok(_) => OpenRgbProbe {
            status: "reachable".into(),
            detail: "127.0.0.1:6742 accepted a TCP connection".into(),
        },
        Err(err) => OpenRgbProbe {
            status: "unreachable".into(),
            detail: err.to_string(),
        },
    }
}
