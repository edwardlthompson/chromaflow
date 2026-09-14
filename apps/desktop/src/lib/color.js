/** HSV/RGB helpers, suggested chips, and last two manual colors. */

export const SUGGESTED_HEX = [
  "#000000",
  "#ff0000",
  "#ffff00",
  "#00ff00",
  "#00ffff",
  "#0000ff",
  "#ff00ff",
  "#ffffff",
];

const RECENT_KEY = "chromaflow.recent-colors";

export function normalizeHex(raw) {
  const m = String(raw || "")
    .trim()
    .match(/^#?([0-9a-fA-F]{6})$/);
  return m ? `#${m[1].toLowerCase()}` : "";
}

function clampByte(n) {
  return Math.max(0, Math.min(255, Math.round(n)));
}

export function rgbToHex(r, g, b) {
  const c = (n) => clampByte(n).toString(16).padStart(2, "0");
  return `#${c(r)}${c(g)}${c(b)}`;
}

export function hexToRgb(raw) {
  const hex = normalizeHex(raw);
  if (!hex) return null;
  return {
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16),
  };
}

export function rgbToHsv(r, g, b) {
  const rr = r / 255;
  const gg = g / 255;
  const bb = b / 255;
  const max = Math.max(rr, gg, bb);
  const min = Math.min(rr, gg, bb);
  const d = max - min;
  let h = 0;
  if (d) {
    if (max === rr) h = ((gg - bb) / d) % 6;
    else if (max === gg) h = (bb - rr) / d + 2;
    else h = (rr - gg) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  return { h, s: max ? d / max : 0, v: max };
}

export function hsvToRgb(h, s, v) {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  let r = 0;
  let g = 0;
  let b = 0;
  if (h < 60) [r, g, b] = [c, x, 0];
  else if (h < 120) [r, g, b] = [x, c, 0];
  else if (h < 180) [r, g, b] = [0, c, x];
  else if (h < 240) [r, g, b] = [0, x, c];
  else if (h < 300) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  return { r: (r + m) * 255, g: (g + m) * 255, b: (b + m) * 255 };
}

export function hsvToHex(h, s, v) {
  const rgb = hsvToRgb(h, s, v);
  return rgbToHex(rgb.r, rgb.g, rgb.b);
}

/** Same 16-bit rainbow as Rust lighting_cycle::rgb16. Local preview only. */
export function periodMs(speed) {
  const s = Math.max(8, Number(speed) || 128);
  return Math.min(40000, Math.max(16000, Math.floor(2500000 / s)));
}

export function phase16(nowMs, speed) {
  const period = Math.max(1, periodMs(speed));
  const ms = Math.floor(Number(nowMs) % period);
  return Math.floor((ms * 65536) / period) & 65535;
}

export function rgb16(h) {
  const x = (h & 65535) * 6;
  const region = Math.floor(x / 65536);
  const t = Math.floor(((x % 65536) * 255) / 65535);
  const q = 255 - t;
  if (region === 0) return [255, t, 0];
  if (region === 1) return [q, 255, 0];
  if (region === 2) return [0, 255, t];
  if (region === 3) return [0, q, 255];
  if (region === 4) return [t, 0, 255];
  return [255, 0, q];
}

export function hexToHsv(raw) {
  const rgb = hexToRgb(raw);
  return rgb ? rgbToHsv(rgb.r, rgb.g, rgb.b) : null;
}

function memory() {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
}

export function loadRecent() {
  const s = memory();
  if (!s) return [];
  try {
    const raw = JSON.parse(s.getItem(RECENT_KEY) || "[]");
    return (Array.isArray(raw) ? raw : []).map(normalizeHex).filter(Boolean).slice(0, 2);
  } catch {
    return [];
  }
}

export function pushRecent(raw) {
  const hex = normalizeHex(raw);
  const prev = loadRecent().filter((c) => c !== hex);
  const next = hex ? [hex, ...prev].slice(0, 2) : prev;
  const s = memory();
  if (s) s.setItem(RECENT_KEY, JSON.stringify(next));
  return next;
}
