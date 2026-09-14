//! Protocol-0 OpenRGB controller DATA: name, modes, LED names, matrix, color.

use crate::types::RgbDevice;

struct Walk {
    leds: u16,
    modes: Vec<String>,
    blobs: Vec<Vec<u8>>,
    active_mode: i32,
    led_names: Vec<String>,
    grid_w: u16,
    grid_h: u16,
    grid: Vec<i32>,
    color: String,
    led_colors: Vec<String>,
}

pub fn parse_controller(data: &[u8]) -> Result<RgbDevice, String> {
    parse_controller_at(data, 0)
}

pub fn parse_controller_at(data: &[u8], proto: u32) -> Result<RgbDevice, String> {
    let name = crate::openrgb_proto::parse_name(data).unwrap_or_default();
    let w = walk_device(data, proto)?;
    Ok(RgbDevice {
        name,
        protocol: "OpenRGB SDK".into(),
        leds: w.leds,
        color: w.color,
        modes: w.modes,
        active_mode: w.active_mode,
        led_names: w.led_names,
        grid_w: w.grid_w,
        grid_h: w.grid_h,
        grid: w.grid,
        led_colors: w.led_colors,
    })
}

pub fn parse_led_count(data: &[u8]) -> Result<u16, String> {
    Ok(walk_device(data, 0)?.leds)
}

pub fn parse_led_count_at(data: &[u8], proto: u32) -> Result<u16, String> {
    Ok(walk_device(data, proto)?.leds)
}

pub fn find_mode(data: &[u8], name: &str) -> Result<(usize, Vec<u8>), String> {
    find_mode_at(data, name, 0)
}

pub fn find_mode_at(data: &[u8], name: &str, proto: u32) -> Result<(usize, Vec<u8>), String> {
    let want = name.trim().to_ascii_lowercase();
    if want.is_empty() {
        return Err("mode required".into());
    }
    let w = walk_device(data, proto)?;
    w.modes
        .iter()
        .position(|m| m.to_ascii_lowercase() == want)
        .and_then(|idx| w.blobs.get(idx).cloned().map(|blob| (idx, blob)))
        .ok_or_else(|| format!("OpenRGB has no mode named {name}"))
}

fn mode_ints(proto: u32) -> usize {
    if proto >= 3 {
        48
    } else {
        36
    }
}

pub fn paint_mode_colors(blob: &mut [u8], rgb: [u8; 3]) {
    paint_mode_at(blob, 0, rgb);
}

pub fn tune_mode(blob: &mut [u8], proto: u32, rgb: [u8; 3]) {
    paint_mode_at(blob, proto, rgb);
    fill_motion(blob, proto);
}

pub fn fill_motion(blob: &mut [u8], proto: u32) {
    let nlen = u16::from_le_bytes(
        blob.get(0..2)
            .and_then(|s| s.try_into().ok())
            .unwrap_or([0, 0]),
    ) as usize;
    let base = 2usize.saturating_add(nlen);
    let smin = read_u32(blob, base + 8);
    let smax = read_u32(blob, base + 12);
    let speed_off = if proto >= 3 { base + 32 } else { base + 24 };
    let speed = read_u32(blob, speed_off);
    if speed == 0 && smax > 0 {
        patch_u32(
            blob,
            speed_off,
            if smax > smin { (smin + smax) / 2 } else { smax },
        );
    }
    if proto >= 3 {
        let bmin = read_u32(blob, base + 16);
        let bmax = read_u32(blob, base + 20);
        let bright_off = base + 36;
        if read_u32(blob, bright_off) == 0 && bmax > 0 {
            patch_u32(blob, bright_off, bmax.max(bmin));
        }
    }
}

fn read_u32(blob: &[u8], off: usize) -> u32 {
    blob.get(off..off + 4)
        .and_then(|s| s.try_into().ok())
        .map(u32::from_le_bytes)
        .unwrap_or(0)
}

fn patch_u32(blob: &mut [u8], off: usize, v: u32) {
    if let Some(slot) = blob.get_mut(off..off + 4) {
        slot.copy_from_slice(&v.to_le_bytes());
    }
}

fn paint_mode_at(blob: &mut [u8], proto: u32, rgb: [u8; 3]) {
    if blob.len() < 2 {
        return;
    }
    let nlen = u16::from_le_bytes(blob[0..2].try_into().unwrap_or([0, 0])) as usize;
    let mut i = 2usize.saturating_add(nlen).saturating_add(mode_ints(proto));
    if i + 2 > blob.len() {
        return;
    }
    let n = u16::from_le_bytes(blob[i..i + 2].try_into().unwrap_or([0, 0])) as usize;
    i += 2;
    for _ in 0..n {
        if i + 3 > blob.len() {
            break;
        }
        blob[i] = rgb[0];
        blob[i + 1] = rgb[1];
        blob[i + 2] = rgb[2];
        i += 4;
    }
}

fn walk_device(data: &[u8], proto: u32) -> Result<Walk, String> {
    let mut i = 0usize;
    skip(data, &mut i, 8)?;
    let strings = if proto >= 1 { 6 } else { 5 };
    for _ in 0..strings {
        skip_str(data, &mut i)?;
    }
    let num_modes = u16_at(data, &mut i)?;
    if num_modes > 64 {
        return Err("too many modes".into());
    }
    let active_mode = i32_at(data, &mut i)?;
    let mut modes = Vec::new();
    let mut blobs = Vec::new();
    for _ in 0..num_modes {
        let (name, blob) = read_mode(data, &mut i, proto)?;
        modes.push(name);
        blobs.push(blob);
    }
    let num_zones = u16_at(data, &mut i)?;
    if num_zones > 64 {
        return Err("too many zones".into());
    }
    let mut grid_w = 0u16;
    let mut grid_h = 0u16;
    let mut grid = Vec::new();
    for _ in 0..num_zones {
        if let Some((h, w, cells)) = read_zone(data, &mut i)? {
            if grid.is_empty() {
                grid_h = h;
                grid_w = w;
                grid = cells;
            }
        }
    }
    let leds = u16_at(data, &mut i)?;
    if leds > 2048 {
        return Err("implausible LED count".into());
    }
    let mut led_names = Vec::new();
    let mut names_ok = true;
    for _ in 0..leds {
        match read_str(data, &mut i) {
            Ok(name) => {
                skip(data, &mut i, 4)?;
                led_names.push(name);
            }
            Err(_) => {
                names_ok = false;
                led_names.clear();
                break;
            }
        }
    }
    let led_colors = if names_ok {
        read_colors(data, &mut i).unwrap_or_default()
    } else {
        Vec::new()
    };
    let color = led_colors.first().cloned().unwrap_or_default();
    Ok(Walk {
        leds,
        modes,
        blobs,
        active_mode,
        led_names,
        grid_w,
        grid_h,
        grid,
        color,
        led_colors,
    })
}

fn read_colors(data: &[u8], i: &mut usize) -> Result<Vec<String>, String> {
    let num_colors = u16_at(data, i)?;
    if num_colors == 0 {
        return Ok(Vec::new());
    }
    if num_colors > 2048 {
        return Err("implausible color count".into());
    }
    let mut out = Vec::with_capacity(usize::from(num_colors));
    for _ in 0..num_colors {
        if *i + 3 > data.len() {
            return Err("eof".into());
        }
        out.push(format!(
            "#{:02x}{:02x}{:02x}",
            data[*i],
            data[*i + 1],
            data[*i + 2]
        ));
        *i = (*i + 4).min(data.len());
    }
    Ok(out)
}

fn read_mode(data: &[u8], i: &mut usize, proto: u32) -> Result<(String, Vec<u8>), String> {
    let start = *i;
    let name = read_str(data, i)?;
    skip(data, i, mode_ints(proto))?;
    let colors = u16_at(data, i)?;
    if colors > 256 {
        return Err("too many mode colors".into());
    }
    skip(data, i, 4 * usize::from(colors))?;
    Ok((name, data[start..*i].to_vec()))
}

fn read_zone(data: &[u8], i: &mut usize) -> Result<Option<(u16, u16, Vec<i32>)>, String> {
    skip_str(data, i)?;
    skip(data, i, 16)?;
    let matrix = u16_at(data, i)? as usize;
    if matrix > 1_048_576 {
        return Err("matrix too large".into());
    }
    if matrix < 8 {
        skip(data, i, matrix)?;
        return Ok(None);
    }
    let h = u32_at(data, i)?;
    let w = u32_at(data, i)?;
    let cells = (matrix - 8) / 4;
    if h == 0 || w == 0 || h > 32 || w > 64 || cells != (h as usize) * (w as usize) {
        skip(data, i, matrix.saturating_sub(8))?;
        return Ok(None);
    }
    let mut grid = Vec::with_capacity(cells);
    for _ in 0..cells {
        let v = u32_at(data, i)?;
        grid.push(if v == 0xFFFF_FFFF { -1 } else { v as i32 });
    }
    Ok(Some((h as u16, w as u16, grid)))
}

fn u16_at(data: &[u8], i: &mut usize) -> Result<u16, String> {
    let end = i.checked_add(2).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    let v = u16::from_le_bytes(data[*i..end].try_into().map_err(|_| "u16")?);
    *i = end;
    Ok(v)
}

fn u32_at(data: &[u8], i: &mut usize) -> Result<u32, String> {
    let end = i.checked_add(4).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    let v = u32::from_le_bytes(data[*i..end].try_into().map_err(|_| "u32")?);
    *i = end;
    Ok(v)
}

fn i32_at(data: &[u8], i: &mut usize) -> Result<i32, String> {
    Ok(u32_at(data, i)? as i32)
}

fn skip(data: &[u8], i: &mut usize, n: usize) -> Result<(), String> {
    *i = i.checked_add(n).ok_or_else(|| "eof".to_string())?;
    if *i > data.len() {
        return Err("eof".into());
    }
    Ok(())
}

fn skip_str(data: &[u8], i: &mut usize) -> Result<(), String> {
    let n = u16_at(data, i)? as usize;
    if n > 4096 {
        return Err("string too long".into());
    }
    skip(data, i, n)
}

fn read_str(data: &[u8], i: &mut usize) -> Result<String, String> {
    let n = u16_at(data, i)? as usize;
    if n > 4096 {
        return Err("string too long".into());
    }
    let end = i.checked_add(n).ok_or_else(|| "eof".to_string())?;
    if end > data.len() {
        return Err("eof".into());
    }
    let raw = std::str::from_utf8(&data[*i..end]).unwrap_or("");
    *i = end;
    Ok(raw.trim_end_matches('\0').to_string())
}

#[cfg(test)]
mod tests {
    use super::{find_mode, parse_controller, parse_led_count};

    fn blob(leds: u16, color: [u8; 3]) -> Vec<u8> {
        let mut data = vec![0u8; 8];
        for s in [b"Mobo\0".as_slice(), b"\0", b"\0", b"\0", b"\0"] {
            data.extend_from_slice(&(s.len() as u16).to_le_bytes());
            data.extend_from_slice(s);
        }
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        let mode = b"Breathing\0";
        data.extend_from_slice(&(mode.len() as u16).to_le_bytes());
        data.extend_from_slice(mode);
        data.extend_from_slice(&[0u8; 36]);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&[color[0], color[1], color[2], 0]);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(b"K\0");
        data.extend_from_slice(&[0u8; 16]);
        data.extend_from_slice(&24u16.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&leds.to_le_bytes());
        for n in 0..leds {
            let label = format!("Key: {n}\0");
            data.extend_from_slice(&(label.len() as u16).to_le_bytes());
            data.extend_from_slice(label.as_bytes());
            data.extend_from_slice(&0u32.to_le_bytes());
        }
        data.extend_from_slice(&3u16.to_le_bytes());
        data.extend_from_slice(&[color[0], color[1], color[2], 0]);
        data.extend_from_slice(&[0xff, 0, 0, 0]);
        data.extend_from_slice(&[0, 0xff, 0, 0]);
        data
    }

    #[test]
    fn leds_modes_and_matrix() {
        let data = blob(3, [0, 0x52, 0xff]);
        assert_eq!(parse_led_count(&data).unwrap(), 3);
        let dev = parse_controller(&data).unwrap();
        assert_eq!(dev.name, "Mobo");
        assert_eq!(dev.leds, 3);
        assert_eq!(dev.color, "#0052ff");
        assert_eq!(dev.modes, vec!["Breathing"]);
        assert_eq!(dev.led_names[0], "Key: 0");
        assert_eq!(dev.grid_w, 2);
        assert_eq!(dev.grid_h, 2);
        assert_eq!(dev.grid, vec![0, 1, -1, 2]);
        assert_eq!(dev.led_colors[1], "#ff0000");
        let (idx, blob) = find_mode(&data, "breathing").unwrap();
        assert_eq!(idx, 0);
        assert!(blob.len() > 10);
        let name = b"Wave\0";
        let mut mode = Vec::new();
        mode.extend_from_slice(&(name.len() as u16).to_le_bytes());
        mode.extend_from_slice(name);
        mode.extend_from_slice(&0u32.to_le_bytes());
        mode.extend_from_slice(&1u32.to_le_bytes());
        mode.extend_from_slice(&1u32.to_le_bytes());
        mode.extend_from_slice(&3u32.to_le_bytes());
        mode.extend_from_slice(&[0u8; 20]);
        mode.extend_from_slice(&0u16.to_le_bytes());
        crate::openrgb_parse::tune_mode(&mut mode, 0, [1, 2, 3]);
        let off = 2 + name.len() + 24;
        assert_eq!(
            u32::from_le_bytes(mode[off..off + 4].try_into().unwrap()),
            2
        );
    }

    #[test]
    fn led_count_without_color_bytes() {
        let mut data = vec![0u8; 8];
        for s in [b"X\0".as_slice(), b"\0", b"\0", b"\0", b"\0"] {
            data.extend_from_slice(&(s.len() as u16).to_le_bytes());
            data.extend_from_slice(s);
        }
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&3u16.to_le_bytes());
        assert_eq!(parse_led_count(&data).unwrap(), 3);
        let dev = parse_controller(&data).unwrap();
        assert_eq!(dev.leds, 3);
        assert!(dev.color.is_empty());
        assert!(dev.modes.is_empty());
    }
}
