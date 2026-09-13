//! Fusion D_LED1/D_LED2 addressable headers (chassis radiator ARGB).

use crate::openrgb_proto::{write_pkt, RESIZE, UPDATE_ZONE};

const DLED_LEDS: u16 = 32;

pub fn paint(
    stream: &mut std::net::TcpStream,
    idx: u32,
    name: &str,
    rgb: [u8; 3],
) -> Result<u8, String> {
    if !is_fusion(name) {
        return Ok(0);
    }
    let mut n = 0u8;
    for zone in 0i32..2 {
        let mut rsz = [0u8; 8];
        rsz[0..4].copy_from_slice(&zone.to_le_bytes());
        rsz[4..8].copy_from_slice(&i32::from(DLED_LEDS).to_le_bytes());
        write_pkt(stream, idx, RESIZE, &rsz)?;
        write_pkt(
            stream,
            idx,
            UPDATE_ZONE,
            &zone_body(zone as u32, DLED_LEDS, rgb),
        )?;
        n += 1;
    }
    Ok(n)
}

pub fn zone_body(zone: u32, n: u16, rgb: [u8; 3]) -> Vec<u8> {
    let mut body = Vec::with_capacity(10 + 4 * usize::from(n));
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&zone.to_le_bytes());
    body.extend_from_slice(&n.to_le_bytes());
    for _ in 0..n {
        body.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0]);
    }
    let data_size = u32::try_from(body.len()).unwrap_or(0);
    body[0..4].copy_from_slice(&data_size.to_le_bytes());
    body
}

fn is_fusion(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("fusion") || n.contains("aorus") || n.contains("gigabyte")
}

#[cfg(test)]
mod tests {
    use super::zone_body;

    #[test]
    fn zone_size_matches_pkt() {
        let body = zone_body(1, 2, [0, 0x52, 0xff]);
        assert_eq!(body.len(), 18);
        assert_eq!(&body[0..4], &(body.len() as u32).to_le_bytes());
        assert_eq!(&body[4..8], &1u32.to_le_bytes());
        assert_eq!(&body[8..10], &2u16.to_le_bytes());
    }
}
