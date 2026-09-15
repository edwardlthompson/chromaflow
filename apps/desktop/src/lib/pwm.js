/** PWM take-over helpers. Confirm before the watchdog writes. */

import { confirmAsk } from "./confirmDialog.js";

export function canControl(card, conflicts) {
  const pwmOk = Boolean(card && card.pwm && card.pwm.writable && card.pwm.enable_exists);
  if (card && card.backend === "nvidia") return pwmOk;
  return !(conflicts && conflicts.length) && pwmOk;
}

export function sourceParts(src, fallbackChip) {
  const label = src && src.label ? String(src.label) : "";
  const i = label.indexOf(" ");
  if (i < 0) return { source_chip: fallbackChip || label, source_label: "" };
  return { source_chip: label.slice(0, i), source_label: label.slice(i + 1) };
}

export function channelOf(card, src, extras) {
  const path = String((card.chip && card.chip.path) || "");
  const bits = path.split("/").filter(Boolean);
  const chip = (card.chip && card.chip.name) || "";
  const sp = sourceParts(src, chip);
  const x = extras || {};
  const p = x.params || {};
  return {
    chip,
    pwm: card.pwm.name,
    dir: bits[bits.length - 1] || "",
    source_chip: sp.source_chip,
    source_label: sp.source_label,
    enabled: true,
    curve_id: x.curve_id || "balanced",
    temp_id: x.temp_id || (src && src.id) || "",
    kind: x.kind || "fan",
    step_up: Number(p.step_up) || 0,
    step_down: Number(p.step_down) || 0,
    start_pct: Number(p.start_pct) || 0,
    stop_pct: Number(p.stop_pct) || 0,
    offset: Number(p.offset) || 0,
    min_pct: Number(p.min_pct) || 20,
    hysteresis_c: Number(p.hysteresis_c) || 0,
    response_ms: Number(p.response_ms) || 0,
  };
}

export function confirmTakeover(ask) {
  return confirmAsk(ask);
}

export function cardForChannel(cards, ch) {
  const list = cards || [];
  const pwm = ch && ch.pwm;
  if (!pwm) return null;
  const dir = (ch.dir || "").trim();
  const byDir = list.find((c) => c.pwm && c.pwm.name === pwm && (c.chip.path || "").endsWith(dir));
  if (byDir) return byDir;
  if ((ch.chip || "") === "nvidia") {
    return list.find((c) => c.backend === "nvidia" && c.pwm && c.pwm.name === pwm) || null;
  }
  return list.find((c) => c.pwm && c.pwm.name === pwm && (c.chip.name || "") === ch.chip) || null;
}
