/** Green→yellow→red or blue→magenta→red meter colors. */

import { rgbToHex } from "./color.js";

export function tempRatio(celsius) {
  if (celsius == null || celsius === "") return null;
  const c = Number(celsius);
  if (!Number.isFinite(c)) return null;
  return Math.max(0, Math.min(1, (c - 35) / 55));
}

function meanPair(a, b, empty) {
  if (a == null && b == null) return empty;
  if (a == null) return b;
  if (b == null) return a;
  return (a + b) / 2;
}

export function combinedTempRatio(cpuC, gpuC) {
  return meanPair(tempRatio(cpuC), tempRatio(gpuC), 0);
}

export function usageRatio(v) {
  if (v == null || v === "") return null;
  const n = Number(v);
  if (!Number.isFinite(n)) return null;
  return Math.max(0, Math.min(1, n));
}

export function combinedUsageRatio(cpuLoad, gpuLoad) {
  return meanPair(usageRatio(cpuLoad), usageRatio(gpuLoad), null);
}

export function laneRatio(g, lane, metric) {
  const row = g || {};
  const use = metric === "usage";
  if (lane === "gauge_ram") return use ? usageRatio(row.ram) ?? 0 : tempRatio(row.ram_c) ?? 0;
  if (lane === "gauge_disk") return use ? usageRatio(row.disk) ?? 0 : tempRatio(row.disk_c) ?? 0;
  if (lane === "gauge_cpu") return use ? usageRatio(row.cpu_load) ?? 0 : tempRatio(row.cpu_c) ?? 0;
  if (lane === "gauge_gpu") return use ? usageRatio(row.gpu_load) ?? 0 : tempRatio(row.gpu_c) ?? 0;
  return use ? combinedUsageRatio(row.cpu_load, row.gpu_load) ?? 0 : combinedTempRatio(row.cpu_c, row.gpu_c);
}

export function gaugeHex(ratio, palette) {
  const t = Math.max(0, Math.min(1, Number(ratio) || 0));
  const blue = palette === "blue";
  const p = t <= 0.5 ? (blue ? [0, 0, 255] : [0, 255, 0]) : (blue ? [255, 0, 255] : [255, 255, 0]);
  const q = t <= 0.5 ? (blue ? [255, 0, 255] : [255, 255, 0]) : [255, 0, 0];
  const u = t <= 0.5 ? t * 2 : (t - 0.5) * 2;
  return rgbToHex(p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u, p[2] + (q[2] - p[2]) * u);
}

export function combinedCelsius(cpuC, gpuC) {
  const nums = [cpuC, gpuC].filter((n) => n != null && n !== "" && Number.isFinite(Number(n))).map(Number);
  if (!nums.length) return null;
  return nums.reduce((s, n) => s + n, 0) / nums.length;
}

export function fmtTemp(c) {
  if (c == null || c === "" || !Number.isFinite(Number(c))) return "—";
  return `${Number(c).toFixed(1)} °C`;
}

export function fmtPct(ratio) {
  if (ratio == null || ratio === "" || !Number.isFinite(Number(ratio))) return "—";
  return `${Math.round(Number(ratio) * 100)} %`;
}

export function pushSample(hist, g, max = 60) {
  const row = {
    cpu_c: g && g.cpu_c,
    gpu_c: g && g.gpu_c,
    ram: g && g.ram,
    disk: g && g.disk,
    ram_c: g && g.ram_c,
    disk_c: g && g.disk_c,
    cpu_load: g && g.cpu_load,
    gpu_load: g && g.gpu_load,
    c: combinedCelsius(g && g.cpu_c, g && g.gpu_c),
  };
  const next = Array.isArray(hist) ? hist.concat(row) : [row];
  return next.length > max ? next.slice(-max) : next;
}

export function gaugeLane(device) {
  const n = `${(device && device.name) || ""} ${(device && device.backend) || ""}`.toLowerCase();
  if (/geforce|nvidia|4090|liquid|aio|radiator|suprim/.test(n)) return "gpu";
  if (/aorus|fusion|motherboard|x570|gigabyte/.test(n)) return "cpu";
  if ((device && device.backend) === "arena" || /keychron|keyboard|arena|speaker/.test(n)) return "combined";
  return "combined";
}

function channelGroup(name, label) {
  const n = `${name || ""} ${label || ""}`.toLowerCase();
  if (/jc42|spd5118|dimm|sodimm|\bddr[345]\b/.test(n)) return "ram";
  if (/nvme|drivetemp/.test(n)) return "disk";
  if (/amdgpu|nvidia|nouveau/.test(n)) return "gpu";
  if (/k10|coretemp|zenpower/.test(n)) return "cpu";
  return "";
}

export function fromHwmon(hwmon) {
  let cpu_c = null;
  let gpu_c = null;
  let ram_c = null;
  let disk_c = null;
  for (const chip of hwmon || []) {
    for (const row of chip.temps || []) {
      const group = channelGroup(chip && chip.name, row && row.label);
      if (!group) continue;
      const v = Number(row && row.value);
      if (!Number.isFinite(v)) continue;
      const c = v / 1000;
      if (c < -40 || c > 125) continue;
      if (group === "cpu") cpu_c = cpu_c == null ? c : Math.max(cpu_c, c);
      else if (group === "gpu") gpu_c = gpu_c == null ? c : Math.max(gpu_c, c);
      else if (group === "disk") disk_c = disk_c == null ? c : Math.max(disk_c, c);
      else ram_c = ram_c == null ? c : Math.max(ram_c, c);
    }
  }
  let note = "";
  if (cpu_c == null && gpu_c == null) note = "CPU/GPU temp not in hwmon";
  else if (gpu_c == null) note = "GPU temp not in hwmon";
  else if (cpu_c == null) note = "CPU temp not in hwmon";
  return { cpu_c, gpu_c, ram_c, disk_c, note };
}

export function mergeGauges(live, hwmon) {
  const f = fromHwmon(hwmon);
  const g = live || {};
  const cpu_c = g.cpu_c ?? f.cpu_c;
  const gpu_c = g.gpu_c ?? f.gpu_c;
  let note = g.note || "";
  if (!note) {
    if (cpu_c == null && gpu_c == null) note = f.note || "CPU/GPU temp not in hwmon";
    else if (gpu_c == null) note = "GPU temp unavailable";
    else if (cpu_c == null) note = "CPU temp not in hwmon";
  }
  return {
    cpu_c,
    gpu_c,
    ram: g.ram ?? null,
    disk: g.disk ?? null,
    ram_c: g.ram_c ?? f.ram_c,
    disk_c: g.disk_c ?? f.disk_c,
    cpu_load: g.cpu_load ?? null,
    gpu_load: g.gpu_load ?? null,
    note,
  };
}
