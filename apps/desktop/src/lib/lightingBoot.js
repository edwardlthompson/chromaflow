/** One-shot session color re-apply after GUI start. No PWM. */

import { normalizeHex } from "./color.js";
import { classify } from "./effectViz.js";
import { deviceKey, lightingDevices } from "./lighting.js";

const HOST = new Set(["openrgb", "arena", "prime", "liquidctl", "keychron", "msi_gpu"]);

function hostDevices(inventory) {
  return lightingDevices(inventory).filter((d) => HOST.has((d && d.backend) || ""));
}

function isSolidOrMissing(mode) {
  if (!mode) return true;
  return classify(mode) === "solid";
}

/** Shared Solid Color hex when every host device agrees; else "". */
export function sharedSolidHex(devices, lastColor, lastMode) {
  const list = (devices || []).filter((d) => HOST.has((d && d.backend) || ""));
  if (!list.length) return "";
  let shared = "";
  for (const d of list) {
    const key = deviceKey(d);
    if (!isSolidOrMissing((lastMode || {})[key])) return "";
    const hex = normalizeHex((lastColor || {})[key]);
    if (!hex) return "";
    if (!shared) shared = hex;
    else if (shared !== hex) return "";
  }
  return shared;
}

export function reapplyFromInventory(inventory, { lastColor, lastMode } = {}, invoke) {
  const colors = lastColor || {};
  const modes = lastMode || {};
  if (!Object.keys(colors).length && !Object.keys(modes).length) {
    return Promise.resolve(null);
  }
  const devices = hostDevices(inventory);
  if (!devices.length || typeof invoke !== "function") return Promise.resolve(null);

  const shared = sharedSolidHex(devices, colors, modes);
  if (shared) {
    return invoke("lighting_broadcast", { color: shared, mode: "Solid Color" }).catch(() => null);
  }

  const jobs = [];
  for (const d of devices) {
    const key = deviceKey(d);
    const hex = normalizeHex(colors[key]);
    if (!hex) continue;
    const mode = modes[key] || "";
    const kind = classify(mode);
    if (kind === "cycle_all" || kind === "off") continue;
    const payload = {
      backend: d.backend,
      device: d.name,
      color: hex,
      mode: null,
      led: null,
    };
    if (mode && kind !== "solid") payload.mode = mode;
    else if (kind === "solid" || !mode) payload.mode = null;
    jobs.push(invoke("lighting_apply", payload).catch(() => null));
  }
  return jobs.length ? Promise.all(jobs).then(() => null) : Promise.resolve(null);
}

export function scheduleBootReapply({
  invoke,
  isTauri,
  getInventory,
  getSession,
  maxAttempts = 20,
  delayMs = 500,
} = {}) {
  if (typeof isTauri === "function" && !isTauri()) return;
  if (typeof invoke !== "function") return;
  let attempts = 0;
  let done = false;
  const tick = () => {
    if (done) return;
    attempts += 1;
    const inv = typeof getInventory === "function" ? getInventory() : null;
    const devices = hostDevices(inv);
    if (devices.length) {
      done = true;
      const sess = typeof getSession === "function" ? getSession() || {} : {};
      reapplyFromInventory(
        inv,
        { lastColor: sess.lastColor || {}, lastMode: sess.lastMode || {} },
        invoke,
      ).catch(() => {});
      return;
    }
    if (attempts >= maxAttempts) {
      done = true;
      return;
    }
    setTimeout(tick, Math.max(50, Number(delayMs) || 500));
  };
  setTimeout(tick, Math.max(50, Number(delayMs) || 500));
}
