/** Read-only cooling helpers. Never write PWM. */

import {
  curvePoints as curvePts,
  curveDots as curveDotList,
  tunedCurve as tuned,
  defaultCurve as balancedCurve,
} from "./coolingCurves.js";
import { collectTemps } from "./coolingMix.js";

export function milliC(raw) {
  const n = Number(raw);
  if (!Number.isFinite(n)) return { c: null, text: String(raw), invalid: true };
  const c = n / 1000;
  return { c, text: `${c.toFixed(1)} °C`, invalid: c < -40 || c > 125 };
}

export function pwmPercent(raw) {
  const n = Number(raw);
  if (!Number.isFinite(n)) return null;
  return Math.max(0, Math.min(100, Math.round((n / 255) * 100)));
}

export function emptyHeader(card) {
  if (!card || card.backend === "nvidia") return false;
  const n = Number(card.fan && card.fan.value);
  return !Number.isFinite(n) || n <= 0;
}

export function onBoard(card, hidden, shown) {
  if (!card || (hidden && hidden[card.id])) return false;
  if (shown && shown[card.id]) return true;
  return !emptyHeader(card);
}

export function inHiddenRow(card, hidden, shown) {
  if (!card) return false;
  if (hidden && hidden[card.id]) return true;
  return emptyHeader(card) && !(shown && shown[card.id]);
}

export function featuredChip(chips) {
  if (!chips || !chips.length) return null;
  return chips.find((c) => (c.fans && c.fans.length) || (c.pwms && c.pwms.length)) || chips[0];
}

export function groupName(name) {
  const n = String(name || "").toLowerCase();
  if (n.includes("k10") || n.includes("coretemp") || n.includes("zenpower")) return "cpu";
  if (n.includes("amdgpu") || n.includes("nvidia") || n.includes("nouveau")) return "gpu";
  if (n.includes("nvme") || n.includes("drivetemp")) return "storage";
  if (n.includes("nct") || n.includes("it87") || n.includes("it86") || n.includes("it879")) return "board";
  return "other";
}

export function samplePoints(values) {
  if (!values.length) return "0,80 200,80";
  const min = Math.min(...values, 20);
  const max = Math.max(...values, 80);
  const span = max - min || 1;
  return values
    .map((v, i) => {
      const x = values.length === 1 ? 0 : (i / (values.length - 1)) * 200;
      const y = 90 - ((v - min) / span) * 70;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

export function curvePoints(curve) {
  return curvePts(curve);
}

export function curveDots(curve) {
  return curveDotList(curve);
}

export function tunedCurve(minDuty, maxDuty) {
  return tuned(minDuty, maxDuty);
}

export function defaultCurve() {
  return balancedCurve();
}

export function tempSources(chips, gauges, mixes) {
  return collectTemps(chips, gauges, mixes);
}

function fanForPwm(fans, pwmName) {
  const n = String(pwmName || "").replace(/^pwm/i, "");
  if (!/^\d+$/.test(n)) return null;
  return (fans || []).find((f) => {
    const lab = String((f && f.label) || "");
    return lab === `fan${n}_input` || lab === `fan${n}`;
  }) || null;
}

export function controlCards(chips, gpuFans) {
  const out = [];
  for (const chip of chips || []) {
    const pwms = chip.pwms || [];
    const fans = chip.fans || [];
    const temp = (chip.temps || [])[0] || null;
    if (!pwms.length && fans.length) {
      out.push({ id: `${chip.path}-fan0`, chip, pwm: null, fan: fans[0], temp });
    }
    pwms.forEach((pwm) => {
      out.push({
        id: `${chip.path}-${pwm.name}`,
        chip,
        pwm,
        fan: fanForPwm(fans, pwm.name),
        temp,
      });
    });
  }
  for (const fan of gpuFans || []) {
    const pct = Number(fan.percent);
    const duty = Number.isFinite(pct) ? String(Math.round((pct / 100) * 255)) : "0";
    const idx = String(fan.id || "").split(":").pop();
    out.push({
      id: fan.id,
      backend: "nvidia",
      chip: { name: "nvidia", path: "nvidia" },
      pwm: { name: `fan${idx}`, value: duty, enable_exists: true, writable: Boolean(fan.writable) },
      fan: { label: fan.label || `GPU fan ${idx}`, value: String(fan.rpm ?? "") },
      temp: null,
    });
  }
  return out;
}
