//! Live Support → Install all progress (UI bar + ETA). No PWM.

use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportProgress {
    pub index: u32,
    pub total: u32,
    pub id: String,
    pub label: String,
    pub detail: String,
    pub pct: u8,
    pub eta_sec: u32,
    pub phase: String,
}

pub struct ProgressClock {
    total: u32,
    index: u32,
    weights: Vec<u32>,
    weight_done: u32,
    weight_total: u32,
    started: Instant,
    on_emit: Option<Box<dyn FnMut(&SupportProgress) + Send>>,
}

impl ProgressClock {
    pub fn new(ids: &[(&str, u32)], on_emit: Option<Box<dyn FnMut(&SupportProgress) + Send>>) -> Self {
        let weights: Vec<u32> = ids.iter().map(|(_, w)| (*w).max(1)).collect();
        let weight_total = weights.iter().sum::<u32>().max(1);
        Self {
            total: ids.len() as u32,
            index: 0,
            weights,
            weight_done: 0,
            weight_total,
            started: Instant::now(),
            on_emit,
        }
    }

    pub fn begin(&mut self, id: &str, label: &str) {
        self.index = self.index.saturating_add(1);
        self.emit(id, label, &format!("Starting {label}…"), "start");
    }

    pub fn finish(&mut self, id: &str, label: &str, detail: &str, ok: bool) {
        let i = self.index.saturating_sub(1) as usize;
        if let Some(w) = self.weights.get(i) {
            self.weight_done = self.weight_done.saturating_add(*w);
        }
        self.emit(id, label, detail, if ok { "done" } else { "fail" });
    }

    fn emit(&mut self, id: &str, label: &str, detail: &str, phase: &str) {
        let pct = ((self.weight_done as u64 * 100) / self.weight_total as u64).min(100) as u8;
        let elapsed = self.started.elapsed().as_secs_f64().max(0.5);
        let eta_sec = if self.weight_done == 0 {
            self.weight_total
        } else {
            let rate = self.weight_done as f64 / elapsed;
            let left = self.weight_total.saturating_sub(self.weight_done) as f64;
            (left / rate.max(0.05)).round().clamp(0.0, 3600.0) as u32
        };
        let ev = SupportProgress {
            index: self.index.min(self.total),
            total: self.total,
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
            pct: if phase == "done" && self.index >= self.total {
                100
            } else {
                pct
            },
            eta_sec,
            phase: phase.into(),
        };
        let path = crate::paths::runtime_dir().join("support-progress.json");
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string(&ev) {
            let _ = fs::write(path, text);
        }
        if let Some(cb) = self.on_emit.as_mut() {
            cb(&ev);
        }
    }
}

pub fn read_progress() -> Option<SupportProgress> {
    let path = crate::paths::runtime_dir().join("support-progress.json");
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eta_moves_after_first_step() {
        let mut clock = ProgressClock::new(&[("a", 10), ("b", 10)], None);
        clock.begin("a", "A");
        clock.finish("a", "A", "ok", true);
        assert!(clock.weight_done >= 10);
        clock.begin("b", "B");
        let p = read_progress();
        assert!(p.is_some());
        assert_eq!(p.unwrap().id, "b");
    }
}
