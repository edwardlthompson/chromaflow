/** Host-effect cadence. Caps USB/IPC so Cinnamon does not hitch. */

import { classify, effectFrame, hostOpenRgbFrames, stampHostFrames, isHostEffect } from "./effectViz.js";
import { deviceKey } from "./lighting.js";
import { ui } from "./ui.js";

export function wantsCycle(lastMode) {
  return Object.values(lastMode || {}).some((m) => classify(m) === "cycle_all");
}

export function setHostCycle(on, invoke, isTauri) {
  ui.cycleOn = Boolean(on);
  if (typeof isTauri === "function" && isTauri() && typeof invoke === "function") {
    invoke("lighting_cycle", {
      on: ui.cycleOn,
      speed: Math.max(1, Math.min(255, Math.round(Number(ui.effectSpeed) || 128))),
    }).catch(() => {});
  }
}

export function fusionFirmware(mode) {
  const k = classify(mode);
  return k === "solid" || k === "flash";
}

export const MOTION_MS = 100;
export const GAUGE_MS = 500;
export const SNAP_MS = 500;
export const METER_MS = 1000;
export const PREVIEW_MS = 32;

export function isGaugeKind(kind) {
  return String(kind || "").startsWith("gauge");
}

export function hostCadence(lastMode) {
  if (ui.cycleOn) return PREVIEW_MS;
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

export function bumpPaint() {
  ui.pausePoll = true;
  ui.paintGen = (ui.paintGen || 0) + 1;
  return Promise.resolve(ui.paintBusy).catch(() => {});
}

export function uniformMode(devices, lastMode) {
  const host = (devices || []).filter((d) =>
    ["openrgb", "arena", "prime", "liquidctl", "keychron", "msi_gpu"].includes(d.backend),
  );
  if (!host.length) return "";
  const modes = host.map((d) => (lastMode || {})[deviceKey(d)] || "");
  if (modes.some((m) => !isHostEffect(m))) return "";
  const k0 = classify(modes[0]);
  if (!modes.every((m) => classify(m) === k0)) return "";
  if (k0 !== "cycle_all" && k0 !== "breathing" && k0 !== "flash" && !isGaugeKind(k0)) return "";
  return modes[0];
}

export function sharedHex(devices, lastMode, lastColor, now) {
  const mode = uniformMode(devices, lastMode);
  if (!mode) return "";
  const d = (devices || []).find((x) => classify((lastMode || {})[deviceKey(x)]) === classify(mode));
  if (!d) return "";
  const key = deviceKey(d);
  return effectFrame({ ...d, mode }, now, (lastColor && lastColor[key]) || "#0052ff", ui.effectSpeed)[0] || "";
}

function hidFrame(d, lastMode, lastColor, now) {
  const b = d && d.backend;
  if (b !== "prime" && b !== "msi_gpu" && b !== "liquidctl") return null;
  const key = deviceKey(d);
  const mode = (lastMode || {})[key];
  if (!isHostEffect(mode)) return null;
  if (b === "liquidctl" && fusionFirmware(mode)) return null;
  const hex = effectFrame({ ...d, mode }, now, (lastColor && lastColor[key]) || "#0052ff", ui.effectSpeed)[0];
  if (!hex) return null;
  if (b === "liquidctl") {
    return { backend: b, device: "sync", color: hex, mode: "sync", key: "liquidctl:sync", hex };
  }
  return { backend: b, device: d.name, color: hex, mode: b === "liquidctl" ? "sync" : null, key, hex };
}

export function pushHidNow(devices, lastMode, lastColor, invoke, now) {
  const t = now == null ? performance.now() : now;
  const seen = new Set();
  for (const d of devices || []) {
    const row = hidFrame(d, lastMode, lastColor, t);
    if (!row || seen.has(row.key)) continue;
    seen.add(row.key);
    invoke("lighting_apply", { backend: row.backend, device: row.device, color: row.color, mode: row.mode, led: null }).catch(() => {});
  }
}

export function pushFusionNow(devices, lastMode, lastColor, invoke) {
  for (const d of devices || []) {
    if ((d && d.backend) !== "liquidctl") continue;
    const key = deviceKey(d);
    const mode = (lastMode && lastMode[key]) || "";
    if (!fusionFirmware(mode)) continue;
    const hex = String((lastColor && lastColor[key]) || "FFFFFF").replace("#", "");
    invoke("lighting_apply", { backend: "liquidctl", device: d.name, color: hex, mode, led: null }).catch(() => {});
  }
}

export function startLightingTick({ getDevices, getPreview, setPreview, invoke, isTauri }) {
  let stop = false;
  let inflight = false;
  let lastSig = "";
  let lastPaint = "";
  let lastHid = {};
  let lastFusion = 0;
  let timer = 0;
  const my = (ui.tickId += 1);
  const loop = () => {
    if (stop || ui.tickId !== my) return;
    const now = performance.now();
    const onLight = lightActive();
    const devices = onLight ? getDevices() || [] : [];
    const cadence = onLight ? hostCadence(ui.lastMode) : 0;
    const live = onLight && Boolean(ui.liveBoard);
    const mode = uniformMode(devices, ui.lastMode);
    const hex = mode ? sharedHex(devices, ui.lastMode, ui.lastColor, now) : "";
    const frames = !hex && cadence ? hostOpenRgbFrames(devices, ui.lastMode, ui.lastColor, now, ui.effectSpeed) : [];
    const sig = hex || frameSig(frames);
    if ((hex || frames.length) && sig !== lastPaint) {
      lastPaint = sig;
      setPreview(hex ? stampHostFrames(getPreview(), (devices || []).map((d) => ({ name: d.name, colors: [hex] }))) : stampHostFrames(getPreview(), frames));
    }
    if (isTauri() && onLight && !ui.pausePoll && !inflight) {
      const gen = ui.paintGen;
      if (ui.cycleOn) {
        if (now - lastFusion >= MOTION_MS) {
          lastFusion = now;
        inflight = true;
        const work = Promise.resolve()
          .then(async () => {
            if (ui.pausePoll || ui.paintGen !== gen) return;
            const jobs = [];
            const seen = new Set();
            const next = { ...lastHid };
            for (const d of devices || []) {
              if ((d && d.backend) !== "liquidctl") continue;
              const row = hidFrame(d, ui.lastMode, ui.lastColor, now);
              if (!row || seen.has(row.key) || lastHid[row.key] === row.hex) continue;
              seen.add(row.key);
              jobs.push(
                invoke("lighting_apply", { backend: row.backend, device: row.device, color: row.color, mode: row.mode, led: null }).then(() => {
                  next[row.key] = row.hex;
                }),
              );
            }
            lastHid = next;
            await Promise.all(jobs);
          })
          .catch(() => {});
        ui.paintBusy = work.finally(() => {
          inflight = false;
        });
        }
      } else {
      const motion = cadence === MOTION_MS;
      const push = Boolean(frames.length && (motion || sig !== lastSig));
      const snap = !hex && !frames.length && live;
      const want = hex && hex !== lastHid.broadcast;
      if (want || push || snap) {
        inflight = true;
        const work = Promise.resolve()
          .then(async () => {
            if (ui.pausePoll || ui.paintGen !== gen) return;
            if (hex) {
              await invoke("lighting_broadcast", { color: hex, mode });
              lastHid = { broadcast: hex };
              lastSig = hex;
              return;
            }
            const jobs = [];
            const seen = new Set();
            const next = { ...lastHid };
            for (const d of cadence ? devices : []) {
              const row = hidFrame(d, ui.lastMode, ui.lastColor, now);
              if (!row || seen.has(row.key) || lastHid[row.key] === row.hex) continue;
              seen.add(row.key);
              jobs.push(
                invoke("lighting_apply", { backend: row.backend, device: row.device, color: row.color, mode: row.mode, led: null }).then(() => {
                  next[row.key] = row.hex;
                }),
              );
            }
            if (push) {
              lastSig = sig;
              jobs.push(invoke("lighting_sync", { frames }));
            } else if (snap) {
              jobs.push(
                invoke("lighting_sync", { frames: [] }).then((rows) => {
                  if (!stop && ui.tickId === my && Array.isArray(rows)) setPreview(rows);
                }),
              );
            }
            lastHid = next;
            await Promise.all(jobs);
          })
          .catch(() => {});
        ui.paintBusy = work.finally(() => {
          inflight = false;
        });
      }
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
