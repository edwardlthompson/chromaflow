/** Quiet / Balanced / Performance graphs. Never write PWM. */

export const PRESETS = [
  { id: "quiet", points: [[25, 20], [55, 22], [70, 28], [82, 50], [90, 100]] },
  { id: "balanced", points: [[25, 20], [50, 40], [70, 70], [85, 100]] },
  { id: "performance", points: [[25, 35], [45, 60], [60, 85], [75, 100]] },
  { id: "full", points: [[25, 100], [90, 100]] },
];

export function presetPoints(id) {
  const row = PRESETS.find((p) => p.id === id);
  return (row || PRESETS[1]).points.map((pt) => [pt[0], pt[1]]);
}

export function allCurves(custom) {
  const extra = (custom || []).map((row) => ({
    id: row.id,
    label: row.label || row.id,
    points: row.points && row.points.length ? row.points : presetPoints("balanced"),
  }));
  return PRESETS.concat(extra);
}

export function curveLabel(row, t) {
  return (row && row.label) || (t && t[`cooling.${row.id}`]) || (row && row.id) || "";
}

export function defaultCurve() {
  return presetPoints("balanced");
}

export function tunedCurve(minDuty, maxDuty) {
  const lo = Math.max(0, Math.min(100, Number(minDuty) || 0));
  const hi = Math.max(lo, Math.min(100, Number(maxDuty) || 100));
  return presetPoints("balanced").map(([t, p]) => [t, Math.max(lo, Math.min(hi, p))]);
}

const PLOT = { l: 24, t: 12, w: 168, h: 76 };

export function plotX(t) {
  return PLOT.l + ((Number(t) - 25) / 75) * PLOT.w;
}

export function plotY(p) {
  return PLOT.t + (1 - Number(p) / 100) * PLOT.h;
}

export function curvePoints(curve) {
  return (curve || defaultCurve())
    .map(([t, p]) => `${plotX(t).toFixed(1)},${plotY(p).toFixed(1)}`)
    .join(" ");
}

export function curveDots(curve) {
  return (curve || defaultCurve()).map(([t, p]) => ({
    t: Number(t),
    p: Number(p),
    cx: plotX(t),
    cy: plotY(p),
  }));
}

export function cloneCurve(src, label) {
  const pts = (src && src.points && src.points.length ? src.points : presetPoints((src && src.id) || "balanced")).map(
    (pt) => [pt[0], pt[1]],
  );
  return { id: `custom-${Date.now()}`, label: label || "Custom", points: pts };
}
