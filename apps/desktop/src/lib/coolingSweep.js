/** Board-wide duty↔RPM queue. Never silent 0%. Never NVIDIA sysfs PWM. */

import { defaultKind } from "./coolingBoard.js";
import { emptyHeader } from "./cooling.js";
import { canControl } from "./pwm.js";

export function sweepQueue(cards, kinds, hidden, conflicts) {
  const ok = (c) => Boolean(c && c.pwm && c.backend !== "nvidia" && !hidden[c.id] && !emptyHeader(c) && canControl(c, conflicts));
  const fans = (cards || []).filter((c) => ok(c) && (kinds[c.id] || defaultKind(c)) !== "pump");
  const pumps = (cards || []).filter((c) => ok(c) && (kinds[c.id] || defaultKind(c)) === "pump");
  return fans.concat(pumps);
}

export function claimQueue(queue, taken, tempId, curveId, defaultTemp, sources) {
  const nextTaken = { ...taken };
  const nextTemp = { ...tempId };
  const nextCurve = { ...curveId };
  for (const card of queue || []) {
    if (!nextTemp[card.id]) nextTemp[card.id] = defaultTemp(sources);
    if (!nextCurve[card.id]) nextCurve[card.id] = "balanced";
    nextTaken[card.id] = true;
  }
  return { taken: nextTaken, tempId: nextTemp, curveId: nextCurve };
}

export function progressMsg(t, name, n, total) {
  return t["cooling.calibrateProgress"].replace("{name}", name).replace("{n}", String(n)).replace("{total}", String(total));
}

export function doneMsg(t, n, silent) {
  let msg = t["cooling.calibrateDone"].replace("{n}", String(n));
  if (silent && silent.length) msg += ` ${t["cooling.noTach"]} ${silent.join(", ")}`;
  return msg;
}

export async function sweepEach(queue, invoke, channelOf, sourceFor, extras, names, defaultName, t, onProgress) {
  const nextCal = {};
  const silent = [];
  let n = 0;
  for (const card of queue || []) {
    n += 1;
    const label = names[card.id] || defaultName(card);
    onProgress(progressMsg(t, label, n, queue.length), n, queue.length);
    const ch = channelOf(card, sourceFor(card), extras(card));
    const out = await invoke("pwm_calibrate", { dir: ch.dir, pwm: ch.pwm });
    const rows = (out && out.rows) || [];
    const empty = !rows.length || rows[rows.length - 1][1] === 0;
    nextCal[card.id] = empty ? [] : rows;
    if (empty) silent.push(label);
  }
  return { nextCal, silent };
}
