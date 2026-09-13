/** Shared UI flags. No hardware I/O. */

function writable(value) {
  const subs = new Set();
  return {
    set(v) {
      value = v;
      subs.forEach((fn) => fn(v));
    },
    subscribe(fn) {
      subs.add(fn);
      fn(value);
      return () => subs.delete(fn);
    },
  };
}

export const pwmOn = writable(false);

export const ui = {
  pausePoll: false,
  pollMs: 15000,
  page: "Cooling",
  liveBoard: false,
  effectSpeed: 128,
  lastMode: {},
  lastColor: {},
  tickId: 0,
  gauges: { cpu_c: null, gpu_c: null, ram: null, disk: null, ram_c: null, disk_c: null, cpu_load: null, gpu_load: null, note: "" },
  gaugeHist: [],
  gaugeMetric: "temp",
  gaugePalette: "blue",
  scrolling: false,
  scrollTimer: 0,
};
