/** In-app confirm. PWM and Support still require an explicit yes. */

let askFn = null;

export function bindConfirm(fn) {
  askFn = typeof fn === "function" ? fn : null;
}

export function confirmAsk(message) {
  const text = String(message || "");
  if (askFn) return askFn(text);
  return Promise.resolve(false);
}
