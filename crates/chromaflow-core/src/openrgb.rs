//! Localhost OpenRGB SDK probe (documented TCP header only). No vendored C++.

use crate::openrgb_proto::{close, connect, parse_name, write_pkt, COUNT, SET_CLIENT};
use crate::types::{OpenRgbProbe, RgbDevice};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(15);
const NEG: Duration = Duration::from_secs(2);

struct Cache {
    ok: Option<(Instant, OpenRgbProbe)>,
    fail_at: Option<Instant>,
}

static CACHE: Mutex<Cache> = Mutex::new(Cache {
    ok: None,
    fail_at: None,
});
static CONNECTS: AtomicU32 = AtomicU32::new(0);

pub fn probe() -> OpenRgbProbe {
    if !crate::openrgb_spawn::sdk_opt_in() {
        return disabled_probe();
    }
    probe_cached(false)
}

fn disabled_probe() -> OpenRgbProbe {
    OpenRgbProbe {
        status: "disabled".into(),
        detail: "OpenRGB spawn off; native lighting".into(),
        controllers: Vec::new(),
        sandboxed: crate::openrgb_sandbox::sandboxed(),
        engine_missing: false,
    }
}

pub fn probe_fresh() -> OpenRgbProbe {
    probe_cached(true)
}

pub fn probe_connects() -> u32 {
    CONNECTS.load(Ordering::SeqCst)
}

pub fn drop_probe_cache() {
    let mut g = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    *g = Cache {
        ok: None,
        fail_at: None,
    };
}

fn probe_cached(fresh: bool) -> OpenRgbProbe {
    let sandboxed = crate::openrgb_sandbox::sandboxed();
    let now = Instant::now();
    if !fresh {
        let g = CACHE.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((at, probe)) = &g.ok {
            if now.saturating_duration_since(*at) < TTL {
                return probe.clone();
            }
        }
        if let Some(at) = g.fail_at {
            if now.saturating_duration_since(at) < NEG {
                return g
                    .ok
                    .as_ref()
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| unreachable_probe(sandboxed, "SDK probe cached miss"));
            }
        }
    }
    let Some(_sdk) = crate::openrgb_proto::try_lock_sdk() else {
        let g = CACHE.lock().unwrap_or_else(|p| p.into_inner());
        return g
            .ok
            .as_ref()
            .map(|(_, p)| p.clone())
            .unwrap_or_else(|| busy_probe(sandboxed));
    };
    CONNECTS.fetch_add(1, Ordering::SeqCst);
    let result = match connect() {
        Ok(mut stream) => match list_controllers(&mut stream) {
            Ok(controllers) => OpenRgbProbe {
                status: "reachable".into(),
                detail: format!("127.0.0.1:6742 listed {} controller(s)", controllers.len()),
                controllers,
                sandboxed,
                engine_missing: false,
            },
            Err(err) => OpenRgbProbe {
                status: "reachable".into(),
                detail: format!("connected; SDK list failed: {err}"),
                controllers: Vec::new(),
                sandboxed,
                engine_missing: false,
            },
        },
        Err(err) => OpenRgbProbe {
            status: "unreachable".into(),
            detail: err.to_string(),
            controllers: Vec::new(),
            sandboxed,
            engine_missing: crate::openrgb_spawn::resolve_bin().is_none(),
        },
    };
    let mut g = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    if result.status == "reachable" && !result.detail.contains("SDK list failed") {
        g.fail_at = None;
        g.ok = Some((Instant::now(), result.clone()));
    } else {
        g.fail_at = Some(Instant::now());
    }
    result
}

fn busy_probe(sandboxed: bool) -> OpenRgbProbe {
    OpenRgbProbe {
        status: "reachable".into(),
        detail: "SDK busy (host lighting)".into(),
        controllers: Vec::new(),
        sandboxed,
        engine_missing: false,
    }
}

fn unreachable_probe(sandboxed: bool, detail: &str) -> OpenRgbProbe {
    OpenRgbProbe {
        status: "unreachable".into(),
        detail: detail.into(),
        controllers: Vec::new(),
        sandboxed,
        engine_missing: crate::openrgb_spawn::resolve_bin().is_none(),
    }
}

fn list_controllers(stream: &mut std::net::TcpStream) -> Result<Vec<RgbDevice>, String> {
    let mut client = b"ChromaFlow".to_vec();
    client.push(0);
    write_pkt(stream, 0, SET_CLIENT, &client)?;
    let proto = crate::openrgb_proto::negotiate(stream);
    write_pkt(stream, 0, COUNT, &[])?;
    let payload = crate::openrgb_proto::read_pkt(stream)?;
    if payload.len() < 4 {
        return Err("short count payload".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count bytes")?) as usize;
    if n > 64 {
        return Err("implausible controller count".into());
    }
    let mut devices = Vec::new();
    for idx in 0..n {
        let data = crate::openrgb_proto::request_data(stream, idx as u32, proto)?;
        let mut dev =
            crate::openrgb_parse::parse_controller_at(&data, proto).unwrap_or_else(|_| {
                RgbDevice::sdk(parse_name(&data).unwrap_or_else(|| format!("controller {idx}")))
            });
        if dev.name.is_empty() {
            dev.name = format!("controller {idx}");
        }
        devices.push(dev);
    }
    close(stream);
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_cache_skips_reconnect() {
        drop_probe_cache();
        let _ = probe();
        let mut n = probe_connects();
        for _ in 0..8 {
            let _ = probe();
            let m = probe_connects();
            if m == n {
                break;
            }
            n = m;
        }
        let n = probe_connects();
        let _ = probe();
        assert_eq!(probe_connects(), n);
    }
}
