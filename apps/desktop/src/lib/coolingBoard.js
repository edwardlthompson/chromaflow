/** Recipe file + default names. Never write PWM. */

import { sourceParts } from "./pwm.js";

const ITE_X570S_87952 = {
  pwm1: "FAN4",
  pwm2: "FAN5_PUMP",
  pwm3: "FAN6_PUMP",
  pwm4: "FAN7_PUMP",
  pwm5: "FAN8",
};
const ITE_X570S_8689 = {
  pwm1: "CPU_FAN",
  pwm2: "SYS_FAN1",
  pwm3: "SYS_FAN2",
  pwm4: "SYS_FAN3",
  pwm5: "CPU_OPT",
};

export function defaultName(card) {
  if (card && card.backend === "nvidia") {
    return (card.fan && card.fan.label) || "GPU fan";
  }
  const chip = (card && card.chip && card.chip.name) || "hwmon";
  const pwm = card && card.pwm ? card.pwm.name : "fan";
  if (/it87952/i.test(chip) && ITE_X570S_87952[pwm]) return ITE_X570S_87952[pwm];
  if (/it8689/i.test(chip) && ITE_X570S_8689[pwm]) return ITE_X570S_8689[pwm];
  return `${chip} ${pwm}`;
}

export function defaultKind(card) {
  return /pump/i.test(defaultName(card)) ? "pump" : "fan";
}

export function aioMemberIds(cards) {
  return (cards || [])
    .filter((c) => c.backend === "nvidia" || /PUMP/i.test(defaultName(c)))
    .map((c) => c.id);
}

export function expandUnit(card, cards, units) {
  if (!(units && card && units[card.id])) return [card];
  return (cards || []).filter((c) => units[c.id]);
}

export function defaultTempId(sources) {
  const list = sources || [];
  return (list.find((s) => s.id === "gauge:cpu") || list[0] || {}).id || "";
}

export function defaultParams() {
  return {
    step_up: 5,
    step_down: 5,
    start_pct: 20,
    stop_pct: 0,
    offset: 0,
    min_pct: 20,
    hysteresis_c: 3,
    response_ms: 0,
  };
}

export function paramsOf(card, params) {
  return (params && params[card.id]) || defaultParams();
}

export function recipeChannel(card, src, extras) {
  const path = String((card.chip && card.chip.path) || "");
  const bits = path.split("/").filter(Boolean);
  const chip = (card.chip && card.chip.name) || "";
  const sp = sourceParts(src, chip);
  const x = extras || {};
  const p = x.params || defaultParams();
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

export function recipeFile(channels, mixes, names, calibration, allowZero, units, custom) {
  return {
    schema: 2,
    allow_zero: Boolean(allowZero),
    min_duty: 20,
    max_duty: 100,
    channels,
    mixes: mixes || [],
    names: names || {},
    calibration: calibration || {},
    units: units || [],
    custom: custom || [],
  };
}
