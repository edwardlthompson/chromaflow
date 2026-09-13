/** Tauri invoke helper. Falls back with a typed error when IPC is missing. */

export function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

export async function invoke(cmd, args) {
  if (!isTauri()) {
    throw new Error("Tauri IPC is not available in this window");
  }
  const { invoke: call } = await import("@tauri-apps/api/core");
  return call(cmd, args || {});
}
