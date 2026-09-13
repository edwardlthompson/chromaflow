/** Temp ids: hwmon, Lighting gauges, mix. Never write PWM. */

function milli(raw) {
  const n = Number(raw);
  if (!Number.isFinite(n)) return { c: null, text: String(raw), invalid: true };
  const c = n / 1000;
  return { c, text: `${c.toFixed(1)} °C`, invalid: c < -40 || c > 125 };
}

function gaugeTemp(c) {
  if (c == null || c === "" || !Number.isFinite(Number(c))) {
    return { c: null, text: "—", invalid: true };
  }
  const n = Number(c);
  return { c: n, text: `${n.toFixed(1)} °C`, invalid: n < -40 || n > 125 };
}

export function hwmonSources(chips) {
  const out = [];
  for (const chip of chips || []) {
    for (const row of chip.temps || []) {
      out.push({
        id: `hwmon:${chip.name}/${row.label}`,
        label: `${chip.name} ${row.label}`,
        temp: milli(row.value),
      });
    }
  }
  return out;
}

function usagePct(v) {
  if (v == null || v === "" || !Number.isFinite(Number(v))) return { n: null, text: "—" };
  const n = Number(v);
  const pct = n <= 1.5 ? n * 100 : n;
  return { n: pct, text: `${Math.round(Math.max(0, Math.min(100, pct)))} %` };
}

export function gaugeSources(gauges) {
  const g = gauges || {};
  const cpuU = usagePct(g.cpu_load);
  const gpuU = usagePct(g.gpu_load);
  const combU =
    cpuU.n == null && gpuU.n == null
      ? { n: null, text: "—" }
      : usagePct(((cpuU.n ?? gpuU.n) + (gpuU.n ?? cpuU.n)) / ((cpuU.n != null) + (gpuU.n != null)) / 100);
  return [
    { id: "gauge:cpu", label: "CPU", temp: gaugeTemp(g.cpu_c), usage: cpuU },
    { id: "gauge:gpu", label: "GPU", temp: gaugeTemp(g.gpu_c), usage: gpuU },
    { id: "gauge:combined", label: "Combined", temp: gaugeTemp(mean(g.cpu_c, g.gpu_c)), usage: combU },
    { id: "gauge:ram", label: "RAM", temp: gaugeTemp(g.ram_c), usage: usagePct(g.ram) },
    { id: "gauge:disk", label: "Disk", temp: gaugeTemp(g.disk_c), usage: usagePct(g.disk) },
  ];
}

function mean(a, b) {
  const nums = [a, b].filter((n) => n != null && n !== "" && Number.isFinite(Number(n))).map(Number);
  if (!nums.length) return null;
  return nums.reduce((s, n) => s + n, 0) / nums.length;
}

function mixValue(mix, lookup) {
  const vals = (mix.sources || []).map((id) => lookup[id]).filter((n) => n != null && Number.isFinite(n));
  if (!vals.length) return { c: null, text: "—", invalid: true };
  const off = Number(mix.offset) || 0;
  let c = vals[0];
  if (mix.op === "min") c = Math.min(...vals);
  else if (mix.op === "average" || mix.op === "tavg") c = vals.reduce((s, n) => s + n, 0) / vals.length;
  else if (mix.op === "sum") c = vals.reduce((s, n) => s + n, 0);
  else if (mix.op === "subtract") c = vals[0] - vals.slice(1).reduce((s, n) => s + n, 0);
  else if (mix.op === "offset") c = vals[0] + off;
  else c = Math.max(...vals);
  return { c, text: `${c.toFixed(1)} °C`, invalid: c < -40 || c > 125 };
}

export function collectTemps(chips, gauges, mixes) {
  const hw = hwmonSources(chips);
  const gs = gaugeSources(gauges);
  const lookup = {};
  for (const s of hw.concat(gs)) {
    if (s.temp && s.temp.c != null) lookup[s.id] = s.temp.c;
  }
  const mx = (mixes || []).map((m) => ({
    id: `mix:${m.id}`,
    label: m.label || m.id,
    temp: mixValue(m, lookup),
    usage: { n: null, text: "—" },
  }));
  return gs.concat(mx);
}

export const MIX_OPS = ["max", "min", "average", "sum", "subtract", "offset", "tavg"];

export function pickList(mixes) {
  const gs = [
    { id: "gauge:cpu", label: "CPU" },
    { id: "gauge:gpu", label: "GPU" },
    { id: "gauge:combined", label: "Combined" },
    { id: "gauge:ram", label: "RAM" },
    { id: "gauge:disk", label: "Disk" },
  ];
  const mx = (mixes || []).map((m) => ({ id: `mix:${m.id}`, label: m.label || m.id }));
  return gs.concat(mx);
}
