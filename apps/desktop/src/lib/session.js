/** Restore tab and per-device host effects. No hardware I/O. */

import { isTauri, invoke } from "./tauri.js";
import { ui } from "./ui.js";

export const TABS = ["Cooling", "Lighting", "Profiles", "Support"];
const KEY = "chromaflow.session";
const MAX = 64;

function tabOf(v) {
  const s = String(v || "");
  return TABS.includes(s) ? s : "Cooling";
}

function strMap(raw, ok) {
  const out = {};
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return out;
  for (const [k, v] of Object.entries(raw)) {
    if (Object.keys(out).length >= MAX) break;
    const key = String(k || "").slice(0, 120);
    const val = String(v || "").slice(0, 80);
    if (key && ok(val)) out[key] = val;
  }
  return out;
}

export function parseSession(raw) {
  const s = raw && typeof raw === "object" ? raw : {};
  const speed = Math.max(1, Math.min(255, Math.round(Number(s.effectSpeed) || 128)));
  return {
    tab: tabOf(s.tab),
    lastMode: strMap(s.lastMode, (v) => Boolean(v)),
    lastColor: strMap(s.lastColor, (v) => /^#[0-9a-fA-F]{6}$/.test(v)),
    effectSpeed: speed,
    gaugeMetric: s.gaugeMetric === "usage" ? "usage" : "temp",
    gaugePalette: s.gaugePalette === "green" ? "green" : "blue",
  };
}

export function emptySession() {
  return parseSession({});
}

export function loadLocal() {
  try {
    if (typeof localStorage === "undefined") return emptySession();
    return parseSession(JSON.parse(localStorage.getItem(KEY) || "null"));
  } catch {
    return emptySession();
  }
}

export function applySession(s) {
  const next = parseSession(s);
  ui.lastMode = next.lastMode;
  ui.lastColor = next.lastColor;
  ui.effectSpeed = next.effectSpeed;
  ui.gaugeMetric = next.gaugeMetric;
  ui.gaugePalette = next.gaugePalette;
  ui.page = next.tab;
  return next;
}

export function snapshot(tab) {
  return parseSession({
    tab: tab || ui.page,
    lastMode: ui.lastMode,
    lastColor: ui.lastColor,
    effectSpeed: ui.effectSpeed,
    gaugeMetric: ui.gaugeMetric,
    gaugePalette: ui.gaugePalette,
  });
}

export function saveLocal(s) {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(KEY, JSON.stringify(parseSession(s)));
  } catch {
    /* quota */
  }
}

let timer = 0;

export function persistSession(tab) {
  const name = tab && typeof tab === "object" && !Array.isArray(tab) ? tab.tab : tab;
  const s = snapshot(name);
  saveLocal(s);
  if (typeof window !== "undefined") window.clearTimeout(timer);
  if (!isTauri()) return;
  timer = window.setTimeout(() => {
    invoke("session_save", { session: s }).catch(() => {});
  }, 200);
}

export async function loadSession() {
  let s = loadLocal();
  if (isTauri()) {
    try {
      s = parseSession(await invoke("session_load"));
      saveLocal(s);
    } catch {
      /* keep local */
    }
  }
  return applySession(s);
}
