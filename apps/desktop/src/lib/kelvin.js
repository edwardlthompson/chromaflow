/** Approximate blackbody CCT (Kelvin) to sRGB. Invert by matching kelvinToRgb. */

import { rgbToHex } from "./color.js";

export const KELVIN_MIN = 2000;
export const KELVIN_MAX = 6500;

function clampByte(n) {
  return Math.max(0, Math.min(255, Math.round(n)));
}

export function kelvinToRgb(k) {
  const t = Math.max(1000, Math.min(40000, Number(k) || 0)) / 100;
  let r;
  let g;
  let b;
  if (t <= 66) {
    r = 255;
    g = 99.4708025861 * Math.log(t) - 161.1195681661;
    b = t <= 19 ? 0 : 138.5177312231 * Math.log(t - 10) - 305.0447927307;
  } else {
    r = 329.698727446 * (t - 60) ** -0.1332047592;
    g = 288.1221695283 * (t - 60) ** -0.0755148492;
    b = 255;
  }
  return { r: clampByte(r), g: clampByte(g), b: clampByte(b) };
}

export function kelvinToHex(k) {
  const rgb = kelvinToRgb(k);
  return rgbToHex(rgb.r, rgb.g, rgb.b);
}

function dist2(a, r, g, b) {
  const dr = a.r - r;
  const dg = a.g - g;
  const db = a.b - b;
  return dr * dr + dg * dg + db * db;
}

/** Find K on the Tanner curve whose RGB is closest to (r,g,b). */
export function rgbToKelvin(r, g, b) {
  const acc = { k: KELVIN_MAX, d: Infinity };
  function consider(k) {
    const d = dist2(kelvinToRgb(k), r, g, b);
    if (d < acc.d || (d === acc.d && k > acc.k)) {
      acc.d = d;
      acc.k = k;
    }
  }
  for (let k = KELVIN_MIN; k <= KELVIN_MAX; k += 20) consider(k);
  const lo = Math.max(KELVIN_MIN, acc.k - 19);
  const hi = Math.min(KELVIN_MAX, acc.k + 19);
  for (let k = lo; k <= hi; k++) consider(k);
  return acc.k;
}
