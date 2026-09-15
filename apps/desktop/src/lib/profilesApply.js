/** Named profile → curve file + RGB look. Never writes PWM. */

import { controlCards } from "./cooling.js";
import { extrasOf, fileFromState } from "./coolingPersist.js";
import { defaultTempId } from "./coolingBoard.js";
import { PRESETS } from "./coolingCurves.js";
import { pickList } from "./coolingMix.js";
import { canControl, channelOf } from "./pwm.js";

export const CURVE_SETS = PRESETS.map((p) => p.id);

export const RGB_LOOKS = [
  { id: "off", mode: "Solid Color", color: "#000000" },
  { id: "solid", mode: "Solid Color", color: "#0052ff" },
  { id: "breathing", mode: "Breathing", color: "#0052ff" },
  { id: "cycle-all", mode: "Cycle All", color: "#0052ff" },
];

export function rgbLook(id) {
  return RGB_LOOKS.find((row) => row.id === String(id || "")) || null;
}

export function curveOptions(current) {
  const ids = CURVE_SETS.slice();
  if (current && !ids.includes(current)) ids.push(current);
  return ids;
}

export function rgbOptions(current) {
  const ids = RGB_LOOKS.map((row) => row.id);
  if (current && !ids.includes(current)) ids.push(current);
  return ids;
}

export function canApplyNames(curveSet, rgb) {
  return Boolean(curveSet && rgb && rgbLook(rgb));
}

export function withCurveSet(file, curveSet) {
  const src = file && typeof file === "object" ? file : {};
  const channels = (src.channels || []).map((ch) => ({
    ...ch,
    curve_id: curveSet,
    enabled: true,
  }));
  return { ...src, channels };
}

export function needsZeroAsk(file) {
  return ((file && file.channels) || []).some((ch) => ch.enabled && Number(ch.min_pct) === 0);
}

export function fileFromInventory(inventory, curveSet) {
  const cards = controlCards((inventory && inventory.hwmon) || [], inventory && inventory.gpu_fans);
  const conflicts = (inventory && inventory.conflicts) || [];
  const sources = pickList([]);
  const taken = {};
  const curveId = {};
  const tempId = {};
  const kinds = {};
  const params = {};
  for (const card of cards) {
    if (!card.pwm || !canControl(card, conflicts)) continue;
    taken[card.id] = true;
    curveId[card.id] = curveSet;
  }
  return fileFromState(
    cards,
    taken,
    (c) => extrasOf(c, curveId, tempId, kinds, params, sources),
    channelOf,
    (c) => sources.find((s) => s.id === (tempId[c.id] || defaultTempId(sources))) || sources[0],
    [],
    {},
    {},
    false,
    {},
    [],
    {},
    {},
  );
}

export function takeoverFile(loaded, inventory, curveSet) {
  if (loaded && Array.isArray(loaded.channels) && loaded.channels.length) {
    return withCurveSet(loaded, curveSet);
  }
  return fileFromInventory(inventory, curveSet);
}
