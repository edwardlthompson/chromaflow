/** Keychron Q6 HE / ANSI 100% geometry. OpenRGB’s 21-col map is packed 1u cells. */

import { ledLabel } from "./lighting.js";

const FROW = ["Esc", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];
const EXTRA = {
  13: "Scrl",
  14: "Mic",
  15: "Lock",
  16: "○",
  17: "△",
  18: "□",
  19: "✕",
  96: "Win",
  97: "Alt",
  99: "Alt",
  100: "Win",
  101: "Fn",
};

export function keyColSpan(name) {
  const n = ledLabel(name).toLowerCase();
  if (n === "space") return 6;
  if (n === "left shift") return 2;
  if (n === "right shift") return 3;
  if (n === "enter" || n === "backspace" || n === "tab" || n === "caps lock") return 2;
  if (n === "number pad 0") return 2;
  return 1;
}

export function keyCaption(name, idx) {
  if (idx >= 0 && idx <= 12) return FROW[idx];
  if (EXTRA[idx]) return EXTRA[idx];
  return ledLabel(name);
}

function isQ6(device) {
  const n = String((device && device.name) || "").toLowerCase();
  if (/keychron|q6/.test(n) && Number(device.leds) >= 100) return true;
  const w = Number(device && device.grid_w) || 0;
  const h = Number(device && device.grid_h) || 0;
  return w === 21 && h === 6 && (/keychron|q6/.test(n) || Number(device.leds) === 108);
}

function add(cells, names, idx, xU, y, wU, h) {
  cells.push({
    idx,
    x: Math.round(xU * 4),
    y,
    span: Math.round(wU * 4),
    rowSpan: h || 1,
    label: keyCaption(names[idx] || "", idx),
  });
}

function lay(cells, names, y, items) {
  let x = 0;
  for (const it of items) {
    if (typeof it === "string") {
      x += Number(it);
      continue;
    }
    if (Array.isArray(it)) {
      add(cells, names, it[0], x, y, it[1] || 1, it[2] || 1);
      x += it[1] || 1;
      continue;
    }
    add(cells, names, it, x, y, 1, 1);
    x += 1;
  }
}

export function q6heKeys(device) {
  const names = Array.isArray(device && device.led_names) ? device.led_names : [];
  const cells = [];
  lay(cells, names, 0, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, "2", "1", 13, 14, 15, "1", 16, 17, 18, 19]);
  lay(cells, names, 1, [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, [33, 2], "1", 34, 35, 36, "1", 37, 38, 39, 40]);
  lay(cells, names, 2, [[41, 1.5], 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, [54, 1.5], "1", 55, 56, 57, "1", 58, 59, 60, [61, 1, 2]]);
  lay(cells, names, 3, [[62, 1.75], 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, [74, 2.25], "1", "3", "1", 75, 76, 77]);
  lay(cells, names, 4, [[78, 2.25], 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, [89, 2.75], "1", "1", 90, "1", "1", 91, 92, 93, [94, 1, 2]]);
  lay(cells, names, 5, [[95, 1.25], [96, 1.25], [97, 1.25], [98, 6.25], [99, 1.25], [100, 1.25], [101, 1.25], [102, 1.25], "1", 103, 104, 105, "1", [106, 2], 107]);
  return { w: 96, h: 6, cells };
}

export function matrixKeys(device) {
  const w = Number(device && device.grid_w) || 0;
  const h = Number(device && device.grid_h) || 0;
  const grid = Array.isArray(device && device.grid) ? device.grid : [];
  const names = Array.isArray(device && device.led_names) ? device.led_names : [];
  if (!(w > 0 && h > 0 && grid.length === w * h)) return null;
  if (isQ6(device)) return q6heKeys(device);
  const cells = [];
  for (let y = 0; y < h; y++) {
    const row = grid.slice(y * w, (y + 1) * w);
    for (let x = 0; x < w; x++) {
      const idx = row[x];
      if (idx < 0) continue;
      cells.push({ idx, x, y, span: 1, rowSpan: 1, label: keyCaption(names[idx] || "", idx) });
    }
  }
  return { w, h, cells };
}

export function packBoard(n, names) {
  const h = 6;
  const count = Math.max(0, Number(n) || 0);
  const w = Math.max(24, Math.ceil(count / h) || 24);
  const cells = [];
  for (let i = 0; i < count; i++) {
    cells.push({
      idx: i,
      x: i % w,
      y: Math.floor(i / w),
      span: 1,
      rowSpan: 1,
      label: ledLabel((names && names[i]) || ""),
    });
  }
  return { w, h, cells };
}

export function boardLayout(device) {
  if (isQ6(device)) return q6heKeys(device);
  const keys = matrixKeys(device);
  if (keys) return keys;
  const names = Array.isArray(device && device.led_names) ? device.led_names : [];
  const colors = Array.isArray(device && device.led_colors) ? device.led_colors : [];
  const n = Math.max(Number(device && device.leds) || 0, names.length, colors.length);
  if (!n) return null;
  return packBoard(n, names);
}

export function mosaicDevice(devices) {
  const names = [];
  const colors = [];
  for (const d of devices || []) {
    const cols = Array.isArray(d.led_colors) ? d.led_colors : [];
    const nms = Array.isArray(d.led_names) ? d.led_names : [];
    const n = Math.max(Number(d.leds) || 0, cols.length, nms.length);
    for (let i = 0; i < n; i += 1) {
      names.push(nms[i] || "");
      colors.push(cols[i] || (d && d.color) || "");
    }
  }
  return {
    name: "all",
    leds: Math.max(names.length, colors.length),
    led_names: names,
    led_colors: colors,
    grid_w: 0,
    grid_h: 0,
    grid: [],
  };
}
