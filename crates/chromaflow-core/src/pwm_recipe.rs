//! Named Quiet/Balanced/Performance/100% graphs and temp ids. Never writes sysfs.

use crate::gauges::Gauges;
use crate::pwm_curves::{Channel, CurveFile};
use crate::pwm_policy::MIN_PERCENT;
use crate::types::Inventory;

pub fn points(id: &str) -> Vec<(f32, u8)> {
    match id {
        "quiet" => vec![(25.0, 20), (55.0, 22), (70.0, 28), (82.0, 50), (90.0, 100)],
        "performance" => vec![(25.0, 35), (45.0, 60), (60.0, 85), (75.0, 100)],
        "full" => vec![(25.0, 100), (90.0, 100)],
        _ => vec![(25.0, 20), (50.0, 40), (70.0, 70), (85.0, 100)],
    }
}

pub fn points_for(id: &str, file: &CurveFile) -> Vec<(f32, u8)> {
    file.custom
        .iter()
        .find(|row| row.id == id)
        .and_then(|row| {
            let pts: Vec<(f32, u8)> = row
                .points
                .iter()
                .filter(|p| p[0].is_finite())
                .map(|p| (p[0], p[1].round().clamp(0.0, 100.0) as u8))
                .collect();
            (!pts.is_empty()).then_some(pts)
        })
        .unwrap_or_else(|| points(id))
}

pub fn flat_full(id: &str, file: &CurveFile) -> bool {
    let pts = points_for(id, file);
    !pts.is_empty() && pts.iter().all(|p| p.1 == 100)
}

pub fn interp(temp_c: f32, pts: &[(f32, u8)]) -> u8 {
    if pts.is_empty() {
        return MIN_PERCENT;
    }
    if !temp_c.is_finite() {
        return pts[0].1.max(MIN_PERCENT);
    }
    if temp_c <= pts[0].0 {
        return pts[0].1;
    }
    if temp_c >= pts[pts.len() - 1].0 {
        return pts[pts.len() - 1].1;
    }
    for pair in pts.windows(2) {
        if temp_c <= pair[1].0 {
            let span = pair[1].0 - pair[0].0;
            let t = if span <= 0.0 {
                0.0
            } else {
                (temp_c - pair[0].0) / span
            };
            let a = f32::from(pair[0].1);
            let b = f32::from(pair[1].1);
            return (a + (b - a) * t).round() as u8;
        }
    }
    pts[pts.len() - 1].1
}

pub fn milli_c(raw: &str) -> Option<f32> {
    raw.trim().parse::<f32>().ok().map(|n| n / 1000.0)
}

pub fn temp_of(inv: &Inventory, file: &CurveFile, ch: &Channel, g: &Gauges) -> Option<f32> {
    let id = if ch.temp_id.is_empty() {
        format!("hwmon:{}/{}", ch.source_chip, ch.source_label)
    } else {
        ch.temp_id.clone()
    };
    lookup(inv, file, &id, g, 0)
}

fn lookup(inv: &Inventory, file: &CurveFile, id: &str, g: &Gauges, depth: u8) -> Option<f32> {
    if depth > 6 {
        return None;
    }
    if let Some(rest) = id.strip_prefix("hwmon:") {
        let (chip, label) = rest.split_once('/')?;
        return inv
            .hwmon
            .iter()
            .find(|c| c.name == chip)
            .and_then(|c| c.temps.iter().find(|t| t.label == label))
            .and_then(|t| milli_c(&t.value))
            .filter(|c| c.is_finite());
    }
    if id.starts_with("gauge:") {
        return gauge_c(g, id);
    }
    if let Some(mid) = id.strip_prefix("mix:") {
        let mix = file.mixes.iter().find(|m| m.id == mid)?;
        let vals: Vec<f32> = mix
            .sources
            .iter()
            .filter_map(|s| lookup(inv, file, s, g, depth + 1))
            .collect();
        return mix_op(&mix.op, &vals, mix.offset);
    }
    None
}

fn gauge_c(g: &Gauges, kind: &str) -> Option<f32> {
    match kind {
        "gauge:cpu" => g.cpu_c.map(|c| c as f32),
        "gauge:gpu" => g.gpu_c.map(|c| c as f32),
        "gauge:ram" => g.ram_c.map(|c| c as f32),
        "gauge:disk" => g.disk_c.map(|c| c as f32),
        "gauge:combined" => {
            let nums: Vec<f32> = [g.cpu_c, g.gpu_c]
                .into_iter()
                .flatten()
                .map(|c| c as f32)
                .collect();
            if nums.is_empty() {
                None
            } else {
                Some(nums.iter().sum::<f32>() / nums.len() as f32)
            }
        }
        _ => None,
    }
}

fn mix_op(op: &str, vals: &[f32], offset: f32) -> Option<f32> {
    if vals.is_empty() {
        return None;
    }
    let v = match op {
        "min" => vals.iter().copied().fold(f32::INFINITY, f32::min),
        "average" | "tavg" => vals.iter().sum::<f32>() / vals.len() as f32,
        "sum" => vals.iter().sum(),
        "subtract" => vals[0] - vals.iter().skip(1).sum::<f32>(),
        "offset" => vals[0] + offset,
        _ => vals.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    };
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_stays_low_then_ramps() {
        let p = points("quiet");
        assert!(interp(40.0, &p) <= 22);
        assert!(interp(88.0, &p) >= 70);
        assert_eq!(interp(90.0, &points("performance")), 100);
        assert_eq!(interp(50.0, &points("balanced")), 40);
        assert_eq!(interp(25.0, &points("quiet")), 20);
        assert_eq!(interp(25.0, &points("full")), 100);
        assert_eq!(interp(80.0, &points("full")), 100);
        assert!(flat_full("full", &crate::pwm_curves::empty()));
        assert!(!flat_full("quiet", &crate::pwm_curves::empty()));
        let mut file = crate::pwm_curves::empty();
        file.custom.push(crate::pwm_curves::NamedCurve {
            id: "custom-1".into(),
            label: "X".into(),
            points: vec![[40.0, 30.0], [80.0, 90.0]],
        });
        assert_eq!(points_for("custom-1", &file)[0].1, 30);
        assert_eq!(points_for("quiet", &file), points("quiet"));
    }
}
