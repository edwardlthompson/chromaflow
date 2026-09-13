/** Host-effect cadence. Caps USB/IPC so Cinnamon does not hitch. */

import { classify, effectFrame, hostOpenRgbFrames, stampHostFrames, isHostEffect } from "./effectViz.js";
import { deviceKey } from "./lighting.js";
import { pushSample } from "./gauges.js";
import { ui } from "./ui.js";

export const MOTION_MS = 100;
export const GAUGE_MS = 500;
export const SNAP_MS = 500;
export const METER_MS = 1000;

export function isGaugeKind(kind) {
  return String(kind || "").startsWith("gauge");
}

export function hostCadence(lastMode) {
  let motion = false;
  let slow = false;
  for (const m of Object.values(lastMode || {})) {
    const k = classify(m);
    if (isGaugeKind(k) || k === "solid") slow = true;
    else if (k !== "direct" && k !== "off") motion = true;
  }
  if (motion) return MOTION_MS;
  if (slow) return GAUGE_MS;
  return 0;
}

export function frameSig(frames) {
  return (frames || [])
    .map((f) => {
      const c = f.colors || [];
      const mid = c[Math.floor(c.length / 2)] || "";
      return `${f.name}:${c.length}:${c[0] || ""}:${mid}:${c[c.length - 1] || ""}`;
    })
    .join("|");
}

export function lightActive() {
  return ui.page === "Lighting" && !ui.scrolling;
}

function hidJobs(devices, now, invoke, lastHid) {
  const jobs = [];
  const next = { ...lastHid };
  for (const d of devices || []) {
    const b = d.backend;
    if (b !== "arena" && b !== "prime" && b !== "liquidctl") continue;
    const key = deviceKey(d);
    if (!isHostEffect(ui.lastMode[key])) continue;
    const cols = effectFrame(
      { ...d, mode: ui.lastMode[key] },
      now,
      ui.lastColor[key] || "#0052ff",
      ui.effectSpeed,
    );
    const hex = cols[0];
    if (!hex || next[key] === hex) continue;
    next[key] = hex;
    jobs.push(
      invoke("lighting_apply", { backend: d.backend, device: d.name, color: hex, mode: null, led: null }),
    );
  }
  return { jobs, lastHid: next };
}

export function startLightingTick({ getDevices, getPreview, setPreview, setGauges, invoke, isTauri }) {
  let stop = false;
  let inflight = false;
  let lastMeter = 0;
  let lastSig = "";
  let lastPaint = "";
  let lastHid = {};
  let timer = 0;
  const my = (ui.tickId += 1);
  const loop = () => {
    if (stop || ui.tickId !== my) return;
    const now = performance.now();
    const onLight = lightActive();
    const devices = onLight ? getDevices() || [] : [];
    const cadence = onLight ? hostCadence(ui.lastMode) : 0;
    const live = onLight && Boolean(ui.liveBoard);
    if (isTauri() && onLight && !ui.pausePoll && !ui.scrolling && (lastMeter === 0 || now - lastMeter >= METER_MS)) {
      lastMeter = now;
      invoke("hardware_gauges")
        .then((g) => {
          if (stop || ui.tickId !== my || !g || typeof g !== "object") return;
          const next = {
            cpu_c: g.cpu_c ?? null,
            gpu_c: g.gpu_c ?? null,
            ram: g.ram ?? null,
            disk: g.disk ?? null,
            ram_c: g.ram_c ?? null,
            disk_c: g.disk_c ?? null,
            cpu_load: g.cpu_load ?? null,
            gpu_load: g.gpu_load ?? null,
            note: g.note || "",
          };
          ui.gauges = next;
          ui.gaugeHist = pushSample(ui.gaugeHist, next);
          if (setGauges) setGauges(next, ui.gaugeHist);
        })
        .catch(() => {});
    }
    const frames = cadence
      ? hostOpenRgbFrames(devices, ui.lastMode, ui.lastColor, now, ui.effectSpeed)
      : [];
    const sig = frameSig(frames);
    if (frames.length && sig !== lastPaint) {
      lastPaint = sig;
      setPreview(stampHostFrames(getPreview(), frames));
    }
    if (isTauri() && onLight && !ui.pausePoll && !inflight) {
      const motion = cadence === MOTION_MS;
      const push = Boolean(frames.length && (motion || sig !== lastSig));
      const snap = !frames.length && live;
      const hid = hidJobs(cadence ? devices : [], now, invoke, lastHid);
      if (push || snap || hid.jobs.length) {
        inflight = true;
        lastHid = hid.lastHid;
        const jobs = hid.jobs;
        if (push) {
          lastSig = sig;
          jobs.push(invoke("lighting_sync", { frames }).catch(() => {}));
        } else if (snap) {
          jobs.push(
            invoke("lighting_sync", { frames: [] }).then((rows) => {
              if (!stop && ui.tickId === my && Array.isArray(rows)) setPreview(rows);
            }),
          );
        }
        Promise.all(jobs)
          .catch(() => {})
          .finally(() => {
            inflight = false;
          });
      }
    }
    timer = window.setTimeout(loop, cadence || (live ? SNAP_MS : METER_MS));
  };
  timer = window.setTimeout(loop, 0);
  return () => {
    stop = true;
    window.clearTimeout(timer);
  };
}
