/** Full-width 60s sparks. Temp 25–90 °C; usage 0–100%. Never write PWM. */

export function tempBand(c) {
  const n = Number(c);
  if (!Number.isFinite(n)) return null;
  return Math.max(0, Math.min(1, (n - 25) / 65));
}

export function usageBand(v) {
  if (v == null || v === "" || !Number.isFinite(Number(v))) return null;
  const n = Number(v);
  return Math.max(0, Math.min(1, n <= 1.5 ? n : n / 100));
}

function meanBand(a, b) {
  if (a == null && b == null) return null;
  if (a == null) return b;
  if (b == null) return a;
  return (a + b) / 2;
}

export function pick(hist, id, kind) {
  return (hist || []).map((row) => {
    if (!row) return null;
    if (kind === "usage") {
      if (id === "gauge:cpu") return usageBand(row.cpu_load);
      if (id === "gauge:gpu") return usageBand(row.gpu_load);
      if (id === "gauge:ram") return usageBand(row.ram);
      if (id === "gauge:disk") return usageBand(row.disk);
      if (id === "gauge:combined") return meanBand(usageBand(row.cpu_load), usageBand(row.gpu_load));
      return null;
    }
    if (id === "gauge:cpu") return tempBand(row.cpu_c);
    if (id === "gauge:gpu") return tempBand(row.gpu_c);
    if (id === "gauge:ram") return tempBand(row.ram_c);
    if (id === "gauge:disk") return tempBand(row.disk_c);
    if (id === "gauge:combined") return tempBand(row.c);
    return null;
  });
}

export const SPARK_W = 200;
export const SPARK_H = 72;

export function gridXs(w = SPARK_W, secs = 60, every = 15) {
  const n = secs / every;
  const out = [];
  for (let i = 0; i <= n; i += 1) out.push((i / n) * w);
  return out;
}

export function gridYs(h = SPARK_H) {
  return [0, 0.25, 0.5, 0.75, 1].map((t) => h * (1 - t));
}

export const GRID_XS = gridXs();
export const GRID_YS = gridYs();

export function areaPath(values, w = SPARK_W, h = SPARK_H) {
  const nums = (values || []).map((v) => (v == null || !Number.isFinite(Number(v)) ? null : Number(v)));
  if (!nums.some((n) => n != null)) return "";
  const n = Math.max(nums.length, 2);
  let d = `M 0 ${h}`;
  nums.forEach((v, i) => {
    const x = (i / (n - 1)) * w;
    const y = h - Math.max(0, Math.min(1, v == null ? 0 : v)) * h;
    d += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
  });
  d += ` L ${w} ${h} Z`;
  return d;
}

export function sparkId(id) {
  return String(id || "x").replace(/[^a-z0-9]/gi, "");
}
