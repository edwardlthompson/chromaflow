/** Load/save cooling recipe UI state. Never write PWM. */

import { cardForChannel } from "./pwm.js";
import { defaultKind, defaultParams, defaultTempId, recipeFile } from "./coolingBoard.js";

export function extrasOf(card, curveId, tempId, kinds, params, sources) {
  const src = sources.find((s) => s.id === (tempId[card.id] || defaultTempId(sources))) || sources[0];
  return {
    curve_id: curveId[card.id] || "balanced",
    temp_id: (src && src.id) || defaultTempId(sources),
    kind: kinds[card.id] || defaultKind(card),
    params: params[card.id] || defaultParams(),
  };
}

export function fileFromState(cards, taken, extrasFn, channelOf, sourceFor, mixes, names, calibration, allowZero, units, custom, hidden, shown) {
  const channels = (cards || [])
    .filter((c) => c.pwm)
    .map((c) => {
      const ch = channelOf(c, sourceFor(c), extrasFn(c));
      ch.enabled = Boolean(taken[c.id]);
      return ch;
    });
  const members = Object.keys(units || {}).filter((id) => units[id]);
  const unitRows = members.length ? [{ id: "aio", label: "AIO", members }] : [];
  const file = recipeFile(channels, mixes, names, calibration, allowZero, unitRows, custom);
  file.hidden = hidden || {};
  file.shown = shown || {};
  return file;
}

export function stateFromFile(file, cards) {
  const names = (file && file.names) || {};
  const mixes = (file && file.mixes) || [];
  const custom = (file && file.custom) || [];
  const calibration = (file && file.calibration) || {};
  const hidden = (file && file.hidden) || {};
  const shown = (file && file.shown) || {};
  const taken = {};
  const kinds = {};
  const curveId = {};
  const tempId = {};
  const params = {};
  for (const ch of (file && file.channels) || []) {
    const card = cardForChannel(cards, ch);
    if (!card) continue;
    taken[card.id] = Boolean(ch.enabled);
    kinds[card.id] = ch.kind || "fan";
    curveId[card.id] = ch.curve_id || "balanced";
    tempId[card.id] = ch.temp_id || "";
    params[card.id] = {
      step_up: ch.step_up || 0,
      step_down: ch.step_down || 0,
      start_pct: ch.start_pct || 0,
      stop_pct: ch.stop_pct || 0,
      offset: ch.offset || 0,
      min_pct: ch.min_pct || 20,
      hysteresis_c: ch.hysteresis_c || 0,
      response_ms: ch.response_ms || 0,
    };
  }
  const members = (((file && file.units) || [])[0] || {}).members || [];
  const units = members.length ? Object.fromEntries(members.map((id) => [id, true])) : null;
  return { names, mixes, custom, calibration, hidden, shown, taken, kinds, curveId, tempId, params, units };
}

export function canApplyRecipe(live, recipe, cards, hydrated) {
  return Boolean(live && recipe && (cards || []).length && !hydrated);
}

export function applyPreset(queue, curveId, taken, tempId, defaultTemp, presetId) {
  const nextCurve = { ...curveId };
  const nextTaken = { ...taken };
  const nextTemp = { ...tempId };
  for (const card of queue || []) {
    nextCurve[card.id] = presetId;
    nextTaken[card.id] = true;
    if (!nextTemp[card.id]) nextTemp[card.id] = defaultTemp;
  }
  return { curveId: nextCurve, taken: nextTaken, tempId: nextTemp };
}
