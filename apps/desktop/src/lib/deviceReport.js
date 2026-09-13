/** Build a GitHub device issue. Body is data (LLM01). No PAT. */

export const REPO = "edwardlthompson/chromaflow";
export const MAX_QUERY_CHARS = 1800;
export const DEVICE_ISSUES = `https://github.com/${REPO}/issues?q=is:issue+label:device+is:open`;

export function sanitize(text, home) {
  let s = String(text || "");
  if (home) s = s.split(home).join("~");
  s = s.replace(/\/home\/[^/]+/g, "~");
  s = s.replace(/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/g, "[redacted-email]");
  s = s.replace(/\S+\.env\b/gi, "[redacted-env]");
  return s;
}

export function buildMarkdown(row, opts) {
  const o = opts || {};
  const lines = [
    `name: ${(row && row.name) || ""}`,
    `vid_pid: ${row && row.vendor_id && row.product_id ? `${row.vendor_id}:${row.product_id}` : ""}`,
    `hid_name: ${(row && row.hid_name) || ""}`,
    `kind: ${(row && row.kind) || ""}`,
    `path: ${sanitize((row && row.path) || "", o.home)}`,
    `readable: ${row && row.readable ? "yes" : "no"}`,
    `reason: ${(row && row.reason) || ""}`,
    `kernel: ${o.kernel || ""}`,
    `manufacturer: ${sanitize((row && row.manufacturer) || "", o.home)}`,
    `product: ${sanitize((row && row.product) || "", o.home)}`,
  ];
  if (o.serial) lines.push(`serial: ${o.serial}`);
  if (o.notes) lines.push(`notes: ${sanitize(o.notes, o.home)}`);
  return lines.join("\n");
}

export function issueForm(markdown, title) {
  const base = `https://github.com/${REPO}/issues/new`;
  const params = new URLSearchParams();
  params.set("template", "device.yml");
  params.set("title", title || "[device]:");
  params.set("labels", "device,enhancement");
  params.set("report", markdown);
  const full = `${base}?${params.toString()}`;
  if (full.length <= MAX_QUERY_CHARS) {
    return { url: full, clipboard: markdown, bodyTooLarge: false };
  }
  const short = new URLSearchParams();
  short.set("template", "device.yml");
  short.set("title", title || "[device]:");
  return { url: `${base}?${short.toString()}`, clipboard: markdown, bodyTooLarge: true };
}

export async function submitDeviceReport(row, opts) {
  const o = opts || {};
  const pack = issueForm(buildMarkdown(row, o), `[device]: ${(row && row.name) || "unknown"}`);
  if (typeof navigator !== "undefined" && navigator.clipboard && navigator.clipboard.writeText) {
    await navigator.clipboard.writeText(pack.clipboard);
  }
  if (typeof o.openUrl === "function") await o.openUrl(pack.url);
  else if (typeof window !== "undefined") window.open(pack.url, "_blank", "noopener");
  return pack;
}

export const REPORTS_KEY = "chromaflow.reported";

export function reportId(row) {
  if (row && row.vendor_id && row.product_id) return `${row.vendor_id}:${row.product_id}`;
  return String((row && (row.path || row.name)) || "");
}

export function loadReported() {
  try {
    const raw = typeof localStorage === "undefined" ? null : localStorage.getItem(REPORTS_KEY);
    const arr = raw ? JSON.parse(raw) : [];
    return Array.isArray(arr) ? arr.map(String).filter(Boolean).slice(0, 64) : [];
  } catch {
    return [];
  }
}

export function wasReported(id) {
  return Boolean(id) && loadReported().includes(String(id));
}

export function markReported(id) {
  if (!id) return;
  const next = [...new Set([...loadReported(), String(id)])].slice(0, 64);
  try {
    if (typeof localStorage !== "undefined") localStorage.setItem(REPORTS_KEY, JSON.stringify(next));
  } catch {
    /* quota */
  }
}
