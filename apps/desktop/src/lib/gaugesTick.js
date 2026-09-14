/** Shared CPU/GPU/RAM/disk samples for Cooling and Lighting. No PWM. */

import { pushSample } from "./gauges.js";
import { ui } from "./ui.js";

export const GAUGE_POLL_MS = 400;
export const GAUGE_HIST = 150;

export function snapshot(g) {
  const row = g || {};
  return {
    cpu_c: row.cpu_c ?? null,
    gpu_c: row.gpu_c ?? null,
    ram: row.ram ?? null,
    disk: row.disk ?? null,
    ram_c: row.ram_c ?? null,
    disk_c: row.disk_c ?? null,
    cpu_load: row.cpu_load ?? null,
    gpu_load: row.gpu_load ?? null,
    note: row.note || "",
  };
}

export function startGaugeTick({ invoke, isTauri, onSample }) {
  let stop = false;
  let timer = 0;
  const loop = () => {
    if (stop) return;
    const work = async () => {
      if (ui.scrolling) return;
      if (typeof isTauri !== "function" || !isTauri() || typeof invoke !== "function") return;
      try {
        const g = await invoke("hardware_gauges");
        if (stop || !g || typeof g !== "object") return;
        const next = snapshot(g);
        ui.gauges = next;
        ui.gaugeHist = pushSample(ui.gaugeHist, next, GAUGE_HIST);
        if (typeof onSample === "function") onSample(next, ui.gaugeHist);
      } catch {
        /* keep last */
      }
    };
    Promise.resolve(work()).finally(() => {
      if (!stop) timer = window.setTimeout(loop, GAUGE_POLL_MS);
    });
  };
  loop();
  return () => {
    stop = true;
    window.clearTimeout(timer);
  };
}
