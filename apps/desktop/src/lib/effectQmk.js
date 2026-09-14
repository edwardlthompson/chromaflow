/** QMK RGB-matrix-style host effects. Coordinates are 0–224 like firmware `point`. */

import { hsvToHex, rgbToHex, rgb16, phase16 } from "./color.js";
import { q6heKeys } from "./keyboard.js";

export const CX = 112;
export const CY = 32;

export function time8(nowMs, speed = 128) {
  const s = Math.max(1, Math.floor(Number(speed) / 4) || 32);
  return Math.floor((Number(nowMs) * s) / 256) & 255;
}

export function wrap8(n) {
  return ((Math.trunc(n) % 256) + 256) % 256;
}

export function atan8(dy, dx) {
  const a = Math.atan2(dy, dx);
  return wrap8(Math.round(((a + Math.PI) / (2 * Math.PI)) * 256));
}

export function classify(mode) {
  const m = String(mode || "").toLowerCase();
  if (!m || m === "direct" || m === "custom") return "direct";
  if (m === "off") return "off";
  if (/breath/.test(m)) return "breathing";
  if (/chevron/.test(m)) return "chevron";
  if (/pinwheel/.test(m)) return "pinwheel";
  if (/band/.test(m) && /spiral/.test(m)) return "band_spiral";
  if (/spiral/.test(m)) return "spiral";
  if (/out in dual|out-in dual/.test(m)) return "out_in_dual";
  if (/out in|out-in/.test(m)) return "out_in";
  if (/up down|up\/down/.test(m)) return "cycle_ud";
  if (/rainbow beacon/.test(m)) return "rainbow_beacon";
  if (/dual beacon/.test(m) || (m.includes("beacon") && !m.includes("rainbow"))) return "dual_beacon";
  if (/left right|rainbow wave|\brainbow\b/.test(m)) return "cycle_lr";
  if (/cycle all|spectrum|color cycle/.test(m)) return "cycle_all";
  if (/flash|strobe/.test(m)) return "flash";
  if (/jellybean/.test(m)) return "jellybean";
  if (/pixel rain|raindrops|\brain\b/.test(m)) return "rain";
  if (/splash|reactive/.test(m)) return "splash";
  if (/static|solid/.test(m)) return "solid";
  if (/hardware gauges/.test(m)) return "gauge_route";
  if (m === "combined" || m === "cpu + gpu" || m === "cpu + gpu temp") return "gauge_temp";
  if (m === "gpu" || m === "gpu temp") return "gauge_gpu";
  if (m === "cpu" || m === "cpu temp") return "gauge_cpu";
  if (m === "ram" || m === "ram used") return "gauge_ram";
  if (m === "disk" || m === "disk used") return "gauge_disk";
  return "cycle_lr";
}

export function ledPoints(device) {
  const n = Math.max(
    Number(device && device.leds) || 0,
    ((device && device.led_names) || []).length,
    ((device && device.led_colors) || []).length,
  );
  const q6n = String((device && device.name) || "").toLowerCase();
  if (/keychron|q6/.test(q6n) && n >= 100) {
    const { cells, w, h } = q6heKeys(device);
    const count = Math.max(n, 108);
    const pts = Array.from({ length: count }, () => ({ x: CX, y: CY }));
    const xmax = Math.max(1, w - 1);
    const ymax = Math.max(1, h - 1);
    for (const c of cells) {
      if (c.idx >= 0 && c.idx < count) {
        pts[c.idx] = { x: (c.x / xmax) * 224, y: (c.y / ymax) * 64 };
      }
    }
    return pts;
  }
  const pts = Array.from({ length: n }, (_, i) => ({
    x: n > 1 ? (i / (n - 1)) * 224 : CX,
    y: CY,
  }));
  const w = Number(device && device.grid_w) || 0;
  const h = Number(device && device.grid_h) || 0;
  const grid = (device && device.grid) || [];
  if (w > 0 && h > 0 && grid.length === w * h) {
    for (let row = 0; row < h; row += 1) {
      for (let col = 0; col < w; col += 1) {
        const idx = grid[row * w + col];
        if (idx >= 0 && idx < n) {
          pts[idx] = {
            x: w > 1 ? (col / (w - 1)) * 224 : CX,
            y: h > 1 ? (row / (h - 1)) * 64 : CY,
          };
        }
      }
    }
  }
  return pts;
}

function rgb(h8, s, v) {
  return hsvToHex((wrap8(h8) / 255) * 360, s, v);
}

export function paintKind(kind, pts, nowMs, hsv, speed = 128) {
  const time = time8(nowMs, speed);
  const sat = hsv.s;
  const val = hsv.v;
  const hue0 = Math.round((hsv.h / 360) * 255) & 255;
  const ang = (time / 256) * 2 * Math.PI;
  const sin = Math.round(Math.sin(ang) * 127);
  const cos = Math.round(Math.cos(ang) * 127);
  const pulse = Math.abs(Math.sin((time / 256) * Math.PI));
  return pts.map((p) => {
    const dx = p.x - CX;
    const dy = p.y - CY;
    const dist = Math.min(255, Math.round(Math.hypot(dx, dy)));
    switch (kind) {
      case "off":
        return "#000000";
      case "solid":
        return rgb(hue0, sat, val);
      case "breathing":
        return rgb(hue0, sat, val * pulse);
      case "cycle_all":
        return rgbToHex(...rgb16(phase16(nowMs, speed)));
      case "cycle_lr":
        return rgb(p.x - time, sat, val);
      case "cycle_ud":
        return rgb(p.y - time, sat, val);
      case "chevron":
        return rgb(hue0 + Math.abs(p.y - CY) + (p.x - time), sat, val);
      case "out_in":
        return rgb((3 * dist) / 2 + time, sat, val);
      case "out_in_dual":
        return rgb(3 * Math.min(255, Math.round(Math.hypot(CX / 2 - Math.abs(dx), dy))) + time, sat, val);
      case "pinwheel":
        return rgb(atan8(dy, dx) + time, sat, val);
      case "spiral":
        return rgb(dist - time - atan8(dy, dx), sat, val);
      case "band_spiral":
        return rgb(hue0, sat, val * (0.2 + 0.8 * Math.abs(Math.sin(((dist - time) / 256) * 2 * Math.PI))));
      case "dual_beacon":
        return rgb(hue0 + (dy * cos + dx * sin) / 128, sat, val);
      case "rainbow_beacon":
        return rgb(hue0 + (dy * 2 * cos + dx * 2 * sin) / 128, sat, val);
      case "flash":
        return pulse > 0.5 ? rgb(hue0, sat, val) : "#000000";
      case "splash":
        return rgb(hue0, sat, val * Math.max(0, 1 - Math.abs((dist / 80 - pulse * 2) % 2 - 1)));
      default:
        return rgb(p.x - time, sat, val);
    }
  });
}
