/** Host Direct effects + layout uses live device pixels only. */

import { hexToRgb, normalizeHex, rgbToHsv, hsvToHex } from "./color.js";
import { classify, ledPoints, paintKind, time8 } from "./effectQmk.js";
import { gaugeHex, gaugeLane, laneRatio } from "./gauges.js";
import { ui } from "./ui.js";

export { classify, ledPoints, time8 };

export const HOST_EFFECTS = [
  "Solid Color",
  "Breathing",
  "Band Spiral",
  "Cycle All",
  "Cycle Left Right",
  "Cycle Up Down",
  "Rainbow Moving Chevron",
  "Cycle Out In",
  "Cycle Out In Dual",
  "Cycle Pinwheel",
  "Cycle Spiral",
  "Dual Beacon",
  "Rainbow Beacon",
  "Jellybean Raindrops",
  "Pixel Rain",
  "Flashing",
  "Splash",
  "Hardware gauges",
  "CPU",
  "GPU",
  "Combined",
  "RAM",
  "Disk",
];

export function preferMode(announced, hint) {
  const a = String(announced || "");
  const h = String(hint || "");
  if (h && isHostEffect(h)) return h;
  if (h && (!a || /^(direct|custom)$/i.test(a))) return h;
  return a || h;
}

export function isDirectMode(mode) {
  const m = String(mode || "").toLowerCase();
  return !m || m === "direct" || m === "custom";
}

export function isHostEffect(mode) {
  const k = classify(mode);
  return k !== "direct";
}

export function ledCount(device) {
  return Math.max(
    Number(device && device.leds) || 0,
    ((device && device.led_names) || []).length,
    ((device && device.led_colors) || []).length,
  );
}

let rain = [];
let rainStep = -1;

function rainFrame(n, nowMs, kind, val, speed = 128) {
  const gap = Math.max(12, Math.round((kind === "jellybean" ? 40 : 120) * (128 / Math.max(1, Number(speed) || 128))));
  const step = Math.floor(Number(nowMs) / gap);
  if (rain.length !== n) {
    rain = Array.from({ length: n }, () => hsvToHex(Math.random() * 360, 0.7, val));
  }
  if (step !== rainStep) {
    rainStep = step;
    const i = Math.floor(Math.random() * n);
    if (kind === "jellybean") rain[i] = hsvToHex(Math.random() * 360, 0.5 + Math.random() * 0.5, val);
    else rain[i] = Math.random() < 0.5 ? "#000000" : hsvToHex(Math.random() * 360, 0.5 + Math.random() * 0.5, val);
  }
  return rain.slice();
}

export function effectFrame(device, nowMs, hex, speed = 128) {
  const pts = ledPoints(device);
  const n = pts.length;
  const kind = classify(device && device.mode);
  const rgb = hexToRgb(hex) || { r: 0, g: 82, b: 255 };
  const hsv = rgbToHsv(rgb.r, rgb.g, rgb.b);
  const chromatic =
    kind !== "solid" &&
    kind !== "off" &&
    kind !== "breathing" &&
    kind !== "flash" &&
    !String(kind).startsWith("gauge");
  if (chromatic && hsv.v < 0.08) hsv.v = 1;
  if (chromatic && hsv.s < 0.08) hsv.s = 1;
  const spd = Number(speed) || 128;
  if (String(kind).startsWith("gauge")) {
    const g = ui.gauges || {};
    let lane = kind;
    if (kind === "gauge_route") {
      const L = gaugeLane(device);
      lane = L === "gpu" ? "gauge_gpu" : L === "cpu" ? "gauge_cpu" : "gauge_temp";
    }
    const fill = gaugeHex(laneRatio(g, lane, ui.gaugeMetric), ui.gaugePalette);
    const count = Math.max(pts.length, 1);
    return Array.from({ length: count }, () => fill);
  }
  if (kind === "jellybean" || kind === "rain") return rainFrame(n || 1, nowMs, kind, hsv.v, spd);
  return paintKind(kind, pts.length ? pts : [{ x: 112, y: 32 }], nowMs, hsv, spd);
}

export function hostOpenRgbFrames(devices, lastMode, lastColor, nowMs, speed = 128) {
  const out = [];
  for (const d of devices || []) {
    if ((d && d.backend) !== "openrgb") continue;
    const key = `${d.backend}:${d.name}`;
    const mode = lastMode && lastMode[key];
    if (!isHostEffect(mode)) continue;
    const hex = (lastColor && lastColor[key]) || "#0052ff";
    out.push({ name: d.name, colors: effectFrame({ ...d, mode }, nowMs, hex, speed) });
  }
  return out;
}

export function stampHostFrames(rows, frames) {
  const out = Array.isArray(rows) ? rows.map((r) => ({ ...r })) : [];
  for (const f of frames || []) {
    if (!f || !f.name || !Array.isArray(f.colors) || !f.colors.length) continue;
    const i = out.findIndex((r) => String(r.name || "").toLowerCase() === String(f.name).toLowerCase());
    const patch = { name: f.name, led_colors: f.colors, color: f.colors[0] };
    if (i >= 0) out[i] = { ...out[i], ...patch };
    else out.push(patch);
  }
  return out;
}

export function layoutColors(device) {
  const n = ledCount(device);
  const live = ((device && device.led_colors) || []).map(normalizeHex).filter(Boolean);
  if (!n) return live;
  if (live.length >= n) return live.slice(0, n);
  if (live.length) return Array.from({ length: n }, (_, i) => live[i] || live[0]);
  return [];
}
