//! Localhost OpenRGB SDK framing. Packet IDs from the public SDK wiki.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

static SDK: Mutex<()> = Mutex::new(());

pub fn lock_sdk() -> MutexGuard<'static, ()> {
    SDK.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn try_lock_sdk() -> Option<MutexGuard<'static, ()>> {
    match SDK.try_lock() {
        Ok(g) => Some(g),
        Err(std::sync::TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(std::sync::TryLockError::WouldBlock) => None,
    }
}

pub const MAGIC: &[u8; 4] = b"ORGB";
pub const COUNT: u32 = 0;
pub const DATA: u32 = 1;
pub const SET_CLIENT: u32 = 50;
pub const RESIZE: u32 = 1000;
pub const UPDATE_LEDS: u32 = 1050;
pub const UPDATE_ZONE: u32 = 1051;
pub const UPDATE_LED: u32 = 1052;
pub const SET_CUSTOM: u32 = 1100;
pub const UPDATE_MODE: u32 = 1101;
pub const PROTO: u32 = 40;
pub const TIMEOUT: Duration = Duration::from_millis(200);
pub const CLIENT_PROTO: u32 = 3;

pub fn negotiate(stream: &mut TcpStream) -> u32 {
    if write_pkt(stream, 0, PROTO, &CLIENT_PROTO.to_le_bytes()).is_err() {
        return 0;
    }
    match read_pkt(stream) {
        Ok(payload) if payload.len() >= 4 => {
            let server = u32::from_le_bytes(payload[0..4].try_into().unwrap_or([0, 0, 0, 0]));
            server.min(CLIENT_PROTO)
        }
        _ => 0,
    }
}

pub fn request_data(stream: &mut TcpStream, idx: u32, proto: u32) -> Result<Vec<u8>, String> {
    let body = if proto == 0 {
        Vec::new()
    } else {
        proto.to_le_bytes().to_vec()
    };
    write_pkt(stream, idx, DATA, &body)?;
    read_pkt(stream)
}

pub fn connect() -> Result<TcpStream, String> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6742);
    let stream = TcpStream::connect_timeout(&addr, TIMEOUT).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    let _ = stream.set_nodelay(true);
    Ok(stream)
}

pub fn write_pkt(
    stream: &mut TcpStream,
    device: u32,
    pkt_id: u32,
    body: &[u8],
) -> Result<(), String> {
    let mut hdr = [0u8; 16];
    hdr[0..4].copy_from_slice(MAGIC);
    hdr[4..8].copy_from_slice(&device.to_le_bytes());
    hdr[8..12].copy_from_slice(&pkt_id.to_le_bytes());
    hdr[12..16].copy_from_slice(&(body.len() as u32).to_le_bytes());
    stream.write_all(&hdr).map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

pub fn read_pkt(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut hdr = [0u8; 16];
    stream.read_exact(&mut hdr).map_err(|e| e.to_string())?;
    if &hdr[0..4] != MAGIC {
        return Err("bad SDK magic".into());
    }
    let size = u32::from_le_bytes(hdr[12..16].try_into().map_err(|_| "size bytes")?) as usize;
    if size > 1_048_576 {
        return Err("SDK payload too large".into());
    }
    let mut body = vec![0u8; size];
    if size > 0 {
        stream.read_exact(&mut body).map_err(|e| e.to_string())?;
    }
    Ok(body)
}

pub fn open_client() -> Result<(TcpStream, u32), String> {
    let mut stream = connect()?;
    let io = std::time::Duration::from_secs(2);
    let _ = stream.set_read_timeout(Some(io));
    let _ = stream.set_write_timeout(Some(io));
    let mut client = b"ChromaFlow".to_vec();
    client.push(0);
    write_pkt(&mut stream, 0, SET_CLIENT, &client)?;
    let proto = negotiate(&mut stream);
    Ok((stream, proto))
}

pub fn close(stream: &mut TcpStream) {
    // Write-only FIN: Shutdown::Both was RST-ing in-flight UPDATE_LEDS and
    // flooding OpenRGB with "recv_select failed receiving magic" on inventory polls.
    let _ = stream.shutdown(Shutdown::Write);
}

pub fn parse_name(data: &[u8]) -> Option<String> {
    if data.len() < 10 {
        return None;
    }
    let name_len = u16::from_le_bytes(data[8..10].try_into().ok()?) as usize;
    let start: usize = 10;
    let end = start.checked_add(name_len)?;
    if end > data.len() {
        return None;
    }
    let raw = std::str::from_utf8(&data[start..end])
        .ok()?
        .trim_end_matches('\0');
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_name;

    #[test]
    fn parse_controller_name() {
        let mut data = vec![0u8; 8];
        let name = b"Mobo";
        data.extend_from_slice(&(name.len() as u16).to_le_bytes());
        data.extend_from_slice(name);
        assert_eq!(parse_name(&data).as_deref(), Some("Mobo"));
    }
}
