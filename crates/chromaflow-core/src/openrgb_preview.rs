//! Fast OpenRGB LED snapshot plus optional Direct frame push. No PWM.

use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;

use crate::openrgb_apply::update_leds_colors;
use crate::openrgb_parse::parse_controller_at;
use crate::openrgb_proto::{
    close, open_client, parse_name, read_pkt, request_data, write_pkt, COUNT, SET_CUSTOM,
    UPDATE_LEDS,
};
use crate::types::RgbDevice;

const PREVIEW_IO: Duration = Duration::from_millis(400);

#[derive(Serialize, Clone)]
pub struct ColorFrame {
    pub name: String,
    pub color: String,
    pub led_colors: Vec<String>,
    pub mode: String,
    pub active_mode: i32,
}

#[derive(Deserialize, Clone)]
pub struct LedFrame {
    pub name: String,
    pub colors: Vec<String>,
}

struct Session {
    stream: TcpStream,
    proto: u32,
    names: Vec<String>,
    painted: Vec<bool>,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub fn drop_session() {
    crate::keychron_preview::drop_session();
    let mut guard = SESSION.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(mut ses) = guard.take() {
        close(&mut ses.stream);
    }
}

pub fn snapshot() -> Vec<ColorFrame> {
    sync(Vec::new())
}

pub fn sync(frames: Vec<LedFrame>) -> Vec<ColorFrame> {
    let pushing = !frames.is_empty();
    if pushing {
        let _paint = crate::lighting_broadcast::lock();
        let mut rest = Vec::new();
        for frame in &frames {
            if crate::lighting_port::skip_sdk(&frame.name) {
                let _ = crate::keychron_apply::set_hex_colors(&frame.colors);
            } else if crate::lighting_port::arenaish(&frame.name) {
                let _ = crate::arena_apply::set_zones(&frame.colors);
            } else {
                rest.push(frame.clone());
            }
        }
        if rest.is_empty() {
            return written_rows(&frames);
        }
        return sync_sdk(rest, true, frames);
    }
    if !pushing && !crate::lighting_cycle::running() {
        crate::keychron_preview::kick();
    }
    sync_sdk(frames, false, Vec::new())
}

fn sync_sdk(frames: Vec<LedFrame>, pushing: bool, original: Vec<LedFrame>) -> Vec<ColorFrame> {
    let _sdk = crate::openrgb_proto::lock_sdk();
    let mut guard = SESSION.lock().unwrap_or_else(|p| p.into_inner());
    if guard.is_none() {
        match open_client() {
            Ok((stream, proto)) => {
                let _ = stream.set_read_timeout(Some(PREVIEW_IO));
                let _ = stream.set_write_timeout(Some(PREVIEW_IO));
                *guard = Some(Session {
                    stream,
                    proto,
                    names: Vec::new(),
                    painted: Vec::new(),
                });
            }
            Err(_) => return overlay(Vec::new(), true),
        }
    }
    if pushing {
        let pushed = {
            let Some(ses) = guard.as_mut() else {
                return overlay(Vec::new(), true);
            };
            push(ses, &frames)
        };
        return match pushed {
            Ok(()) => written_rows(if original.is_empty() {
                &frames
            } else {
                &original
            }),
            Err(_) => {
                if let Some(mut ses) = guard.take() {
                    close(&mut ses.stream);
                }
                overlay(Vec::new(), true)
            }
        };
    }
    let pulled = {
        let Some(ses) = guard.as_mut() else {
            return overlay(Vec::new(), true);
        };
        pull(ses)
    };
    match pulled {
        Ok(rows) => overlay(rows, true),
        Err(_) => {
            if let Some(mut ses) = guard.take() {
                close(&mut ses.stream);
            }
            overlay(Vec::new(), true)
        }
    }
}

fn written_rows(frames: &[LedFrame]) -> Vec<ColorFrame> {
    frames
        .iter()
        .filter(|f| !f.colors.is_empty())
        .map(|f| ColorFrame {
            name: f.name.clone(),
            color: f.colors.first().cloned().unwrap_or_default(),
            led_colors: f.colors.clone(),
            mode: "Direct".into(),
            active_mode: 0,
        })
        .collect()
}

fn overlay(mut rows: Vec<ColorFrame>, use_hid: bool) -> Vec<ColorFrame> {
    if !use_hid {
        return rows;
    }
    let hid = crate::keychron_preview::latest();
    if hid.len() < 4 {
        return rows;
    }
    let mut found = false;
    for row in &mut rows {
        if row.name.to_ascii_lowercase().contains("keychron") {
            row.led_colors = hid.clone();
            row.color = hid.first().cloned().unwrap_or_default();
            found = true;
        }
    }
    if !found {
        rows.push(ColorFrame {
            name: "Keychron Q6 HE".into(),
            color: hid.first().cloned().unwrap_or_default(),
            led_colors: hid,
            mode: "Direct".into(),
            active_mode: 0,
        });
    }
    rows
}

fn mode_name(dev: &RgbDevice) -> String {
    usize::try_from(dev.active_mode)
        .ok()
        .and_then(|i| dev.modes.get(i).cloned())
        .unwrap_or_default()
}

fn push(ses: &mut Session, frames: &[LedFrame]) -> Result<(), String> {
    fill_names(ses)?;
    for frame in frames {
        if crate::lighting_port::skip_sdk(&frame.name) {
            continue;
        }
        let want = frame.name.trim().to_ascii_lowercase();
        let Some(idx) = ses
            .names
            .iter()
            .position(|n| n.to_ascii_lowercase() == want)
        else {
            continue;
        };
        let mut rgb = Vec::new();
        for hex in &frame.colors {
            if let Ok(c) = crate::lighting_apply::parse_rrggbb(hex) {
                rgb.push(c);
            }
        }
        if rgb.is_empty() || rgb.len() > 2048 {
            continue;
        }
        if ses.painted.get(idx) != Some(&true) {
            write_pkt(&mut ses.stream, idx as u32, SET_CUSTOM, &[])?;
            std::thread::sleep(Duration::from_millis(20));
            if let Some(slot) = ses.painted.get_mut(idx) {
                *slot = true;
            }
        }
        write_pkt(
            &mut ses.stream,
            idx as u32,
            UPDATE_LEDS,
            &update_leds_colors(&rgb),
        )?;
    }
    Ok(())
}

fn fill_names(ses: &mut Session) -> Result<(), String> {
    if !ses.names.is_empty() {
        return Ok(());
    }
    write_pkt(&mut ses.stream, 0, COUNT, &[])?;
    let payload = read_pkt(&mut ses.stream)?;
    if payload.len() < 4 {
        return Err("short count".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count")?) as usize;
    if n > 64 {
        return Err("implausible controller count".into());
    }
    for idx in 0..n {
        let data = request_data(&mut ses.stream, idx as u32, ses.proto)?;
        ses.names
            .push(parse_name(&data).unwrap_or_else(|| format!("controller {idx}")));
        ses.painted.push(false);
    }
    Ok(())
}

fn pull(ses: &mut Session) -> Result<Vec<ColorFrame>, String> {
    write_pkt(&mut ses.stream, 0, COUNT, &[])?;
    let payload = read_pkt(&mut ses.stream)?;
    if payload.len() < 4 {
        return Err("short count payload".into());
    }
    let n = u32::from_le_bytes(payload[0..4].try_into().map_err(|_| "count bytes")?) as usize;
    if n > 64 {
        return Err("implausible controller count".into());
    }
    if ses.painted.len() != n {
        ses.painted = vec![false; n];
        ses.names = vec![String::new(); n];
    }
    let mut rows = Vec::with_capacity(n);
    for idx in 0..n {
        let data = request_data(&mut ses.stream, idx as u32, ses.proto)?;
        let mut dev = parse_controller_at(&data, ses.proto)
            .unwrap_or_else(|_| crate::types::RgbDevice::sdk(format!("controller {idx}")));
        if dev.name.is_empty() {
            dev.name = format!("controller {idx}");
        }
        ses.names[idx] = dev.name.clone();
        let mode = mode_name(&dev);
        if !mode.is_empty()
            && !mode.eq_ignore_ascii_case("direct")
            && !mode.eq_ignore_ascii_case("custom")
        {
            ses.painted[idx] = false;
        }
        rows.push(ColorFrame {
            name: dev.name,
            color: dev.color,
            led_colors: dev.led_colors,
            mode,
            active_mode: dev.active_mode,
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    #[test]
    fn snapshot_never_panics() {
        let _ = super::snapshot();
    }

    #[test]
    fn drop_session_is_idempotent() {
        super::drop_session();
        super::drop_session();
    }

    #[test]
    fn written_rows_keep_host_pixels() {
        let rows = super::written_rows(&[super::LedFrame {
            name: "Keychron Q6 HE".into(),
            colors: vec!["#ff0000".into(), "#00aa00".into()],
        }]);
        assert_eq!(rows[0].led_colors, vec!["#ff0000", "#00aa00"]);
        assert_eq!(rows[0].color, "#ff0000");
        assert_eq!(rows[0].mode, "Direct");
    }

    #[test]
    fn live_host_frame_writes_when_asked() {
        if std::env::var("CHROMAFLOW_LIVE_RGB").ok().as_deref() != Some("1") {
            return;
        }
        super::drop_session();
        let probe = crate::openrgb::probe();
        assert_eq!(probe.status, "reachable", "{}", probe.detail);
        let ctl = probe
            .controllers
            .iter()
            .find(|c| c.name.to_ascii_lowercase().contains("keychron"))
            .expect("keychron on OpenRGB");
        let n = usize::from(ctl.leds.clamp(1, 2048));
        let frame = |shift: u8| -> Vec<String> {
            (0..n)
                .map(|i| {
                    let h = ((i.saturating_mul(255)) / n.max(1)) as u8;
                    let h = h.wrapping_add(shift);
                    format!("#{h:02x}40{:02x}", 255u8.wrapping_sub(h))
                })
                .collect()
        };
        let name = ctl.name.clone();
        if let Ok(path) = std::env::var("CHROMAFLOW_LIVE_CHEVRON") {
            let payload: Vec<Vec<String>> =
                serde_json::from_str(&std::fs::read_to_string(path).expect("chevron json"))
                    .expect("chevron frames");
            assert!(payload.len() >= 4, "need several chevron frames");
            let mut last = String::new();
            for colors in payload {
                assert_eq!(colors.len(), n);
                assert_ne!(colors[0], "#ffffff");
                assert_ne!(colors[0], "#000000");
                if !last.is_empty() {
                    assert_ne!(colors[0], last, "chevron must move");
                }
                last = colors[0].clone();
                let rows = super::sync(vec![super::LedFrame {
                    name: name.clone(),
                    colors: colors.clone(),
                }]);
                assert_eq!(rows[0].led_colors, colors);
                std::thread::sleep(std::time::Duration::from_millis(70));
            }
            return;
        }
        let mut last = String::new();
        for (i, delay_ms) in [
            (0u8, 120u64),
            (24, 120),
            (48, 120),
            (72, 120),
            (96, 50),
            (144, 50),
            (192, 50),
            (240, 50),
        ] {
            let colors = frame(i);
            assert_ne!(colors[0], "#ffffff");
            assert_ne!(colors[0], "#000000");
            if !last.is_empty() {
                assert_ne!(colors[0], last);
            }
            last = colors[0].clone();
            let rows = super::sync(vec![super::LedFrame {
                name: name.clone(),
                colors: colors.clone(),
            }]);
            assert_eq!(
                rows[0].led_colors, colors,
                "written pixels must match the host frame"
            );
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }
}
