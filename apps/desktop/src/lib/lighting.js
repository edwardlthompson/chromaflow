/** Group USB/ARGB inventory. Apply uses OpenRGB, liquidctl, Arena 7, and Prime Neo. */

import { normalizeHex } from "./color.js";

export { normalizeHex };

export function hidList(inventory) {
  return Array.isArray(inventory && inventory.hid_rgb) ? inventory.hid_rgb : [];
}

export function liquidctlList(inventory) {
  return Array.isArray(inventory && inventory.liquidctl_devices)
    ? inventory.liquidctl_devices
    : [];
}

export function controllerName(row) {
  if (typeof row === "string") return row;
  return (row && row.name) || "";
}

export function deviceKey(d) {
  return `${(d && d.backend) || ""}:${(d && d.name) || ""}`;
}

export function sizeLabel(d) {
  const n = Number(d && d.leds) || 0;
  if (d && d.backend === "arena") return "4 HID zones";
  if (d && d.backend === "prime") return "wheel LED";
  return n ? `${n} LEDs` : "size unknown";
}

export function radiatorFromOpenRgb(controllers) {
  return (controllers || []).map(controllerName).filter((n) => /liquid|aio|radiator/i.test(n));
}

export function needsDetectionSupport(inventory) {
  const ids = ((inventory && inventory.gaps) || []).map((g) => g.id);
  return ["udev_hidraw", "i2c_dev_missing", "usb_rgb_not_in_openrgb", "openrgb_sandboxed"].some(
    (id) => ids.includes(id),
  );
}

export function sdkTargets(inventory) {
  return ((inventory && inventory.openrgb && inventory.openrgb.controllers) || []).map((row) => ({
    backend: "openrgb",
    name: controllerName(row),
    protocol: (row && row.protocol) || "OpenRGB SDK",
    leds: Number(row && row.leds) || 0,
    color: normalizeHex(row && row.color) || "",
    modes: Array.isArray(row && row.modes) ? row.modes.filter(Boolean) : [],
    led_names: Array.isArray(row && row.led_names) ? row.led_names : [],
    grid_w: Number(row && row.grid_w) || 0,
    grid_h: Number(row && row.grid_h) || 0,
    grid: Array.isArray(row && row.grid) ? row.grid : [],
    led_colors: Array.isArray(row && row.led_colors) ? row.led_colors : [],
    active_mode: Number(row && row.active_mode) || 0,
    mode: Array.isArray(row && row.modes)
      ? String(row.modes[Number(row.active_mode)] || "")
      : "",
  }));
}

export function arenaTargets(inventory) {
  return hidList(inventory)
    .filter((d) => d.readable && d.vendor_id === "1038" && d.product_id === "1a00")
    .map((d) => ({
      backend: "arena",
      name: d.name,
      protocol: "HID report 0x06",
      leds: 4,
      color: "",
      modes: [],
      led_names: [],
      grid_w: 0,
      grid_h: 0,
      grid: [],
    }));
}

export function fusionFallback(inventory) {
  const listed = ((inventory && inventory.openrgb && inventory.openrgb.controllers) || [])
    .map(controllerName)
    .join(" ")
    .toLowerCase();
  const hasSdk = /fusion|aorus|gigabyte/i.test(listed);
  return liquidctlList(inventory)
    .filter((n) => /fusion/i.test(n) && !hasSdk)
    .map((name) => ({
      backend: "liquidctl",
      name,
      protocol: "liquidctl",
      leds: 0,
      color: "",
      modes: [],
      led_names: [],
      grid_w: 0,
      grid_h: 0,
      grid: [],
    }));
}

export function primeTargets(inventory) {
  return hidList(inventory)
    .filter((d) => d.readable && d.vendor_id === "1038" && d.product_id === "1856")
    .map((d) => ({
      backend: "prime",
      name: d.name,
      protocol: "HID 0x62",
      leds: 1,
      color: "",
      modes: [],
      led_names: [],
      grid_w: 0,
      grid_h: 0,
      grid: [],
    }));
}

export function lightingDevices(inventory) {
  return sdkTargets(inventory)
    .concat(arenaTargets(inventory))
    .concat(primeTargets(inventory))
    .concat(fusionFallback(inventory));
}

export { researchDevices, fusionUsbListed, hidHasBackend } from "./research.js";

export const EXTRA_KERNEL_IDS = [
  "it87-dkms",
  "liquidctl",
  "linux-modules-extra",
  "i2c-dev",
  "i2c-piix4",
  "it87",
  "nct6775",
  "k10temp",
  "jc42",
  "spd5118",
  "gigabyte_wmi",
  "udev",
  "group_i2c",
  "group_plugdev",
  "pwm_acl",
  "i2c-nct6775",
  "i2c-nvidia-gpu",
];

export function uniqueModes(devices) {
  const seen = new Set();
  const out = [];
  for (const d of devices || []) {
    for (const name of d.modes || []) {
      if (!seen.has(name)) {
        seen.add(name);
        out.push(name);
      }
    }
  }
  return out;
}

export function ledLabel(name) {
  const s = String(name || "");
  const cut = s.includes(":") ? s.split(":").pop().trim() : s.trim();
  return cut || "·";
}

export function hasLedLayout(d) {
  return Boolean(boardCount(d));
}

export function boardCount(d) {
  const w = Number(d && d.grid_w) || 0;
  const h = Number(d && d.grid_h) || 0;
  const grid = (d && d.grid) || [];
  if (w > 0 && h > 0 && grid.length === w * h) return grid.filter((i) => i >= 0).length || Number(d.leds) || 0;
  return Math.max(Number(d && d.leds) || 0, ((d && d.led_names) || []).length, ((d && d.led_colors) || []).length);
}

export function liveSwatch(d) {
  const cols = ((d && d.led_colors) || []).map(normalizeHex).filter(Boolean);
  if (cols.length >= 2) {
    const step = Math.max(1, Math.floor(cols.length / 8));
    const stops = cols.filter((_, i) => i % step === 0).slice(0, 8);
    return `linear-gradient(90deg, ${stops.join(",")})`;
  }
  return cols[0] || normalizeHex(d && d.color) || "#222";
}

export function mergePreview(devices, preview) {
  const by = {};
  for (const row of preview || []) {
    if (row && row.name) by[row.name] = row;
  }
  return (devices || []).map((d) => {
    const live = by[d.name];
    if (!live) return d;
    const cols = Array.isArray(live.led_colors) ? live.led_colors : [];
    return {
      ...d,
      color: live.color || d.color,
      led_colors: cols.length ? cols : d.led_colors,
      mode: live.mode || d.mode || "",
      active_mode: live.active_mode == null ? d.active_mode : live.active_mode,
    };
  });
}

export function extraFailNote(results, id, present) {
  if (present) return "";
  const row = (results || []).find((r) => r.id === id);
  const msg = row && row.ok === false ? String(row.error || "install failed") : "";
  return msg.length > 180 ? `${msg.slice(0, 180)}…` : msg;
}

export function mergeExtraResults(prev, next) {
  const by = {};
  for (const row of prev || []) by[row.id] = row;
  for (const row of next || []) by[row.id] = row;
  return EXTRA_KERNEL_IDS.map((id) => by[id]).filter(Boolean);
}

export function extraKernelItems(plan) {
  const byId = {};
  for (const row of (plan && plan.extras) || []) {
    byId[row.id] = { present: Boolean(row.present), label: row.label || "" };
  }
  return EXTRA_KERNEL_IDS.map((id) => ({
    id,
    present: Boolean(byId[id] && byId[id].present),
    label: (byId[id] && byId[id].label) || "",
  }));
}
