/** Keep lighting picker cards equal 1×1 squares; grow shared --picker-size to fit. */

export const PICKER_BASE_REM = 16.75;

function basePx() {
  if (typeof window === "undefined" || typeof document === "undefined") return PICKER_BASE_REM * 16;
  const root = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
  return PICKER_BASE_REM * root;
}

export function fitPickerFlow(flow) {
  if (!flow || typeof window === "undefined") return 0;
  const squares = Array.from(flow.querySelectorAll(".picker-square"));
  if (!squares.length) return 0;

  const base = basePx();
  flow.style.setProperty("--picker-size", `${base}px`);

  const saved = squares.map((el) => ({
    el,
    height: el.style.height,
    maxHeight: el.style.maxHeight,
    minHeight: el.style.minHeight,
    overflow: el.style.overflow,
    flex: el.style.flex,
  }));

  let need = base;
  for (const el of squares) {
    el.style.height = "auto";
    el.style.maxHeight = "none";
    el.style.minHeight = "0";
    el.style.overflow = "visible";
    el.style.flex = "0 0 auto";
    need = Math.max(need, el.scrollHeight);
  }

  for (const s of saved) {
    s.el.style.height = s.height;
    s.el.style.maxHeight = s.maxHeight;
    s.el.style.minHeight = s.minHeight;
    s.el.style.overflow = s.overflow;
    s.el.style.flex = s.flex;
  }

  const size = Math.ceil(Math.max(base, need));
  flow.style.setProperty("--picker-size", `${size}px`);
  return size;
}

export function fitOpenPickerFlows(root) {
  if (typeof document === "undefined") return;
  const scope = root && root.querySelectorAll ? root : document;
  scope.querySelectorAll(".picker-slot.is-open .picker-flow").forEach(fitPickerFlow);
}
