/** GitHub update check. One in-flight request. Launch failures stay quiet. */

import { isTauri, invoke } from "./tauri.js";
import { confirmAsk } from "./confirmDialog.js";

let flight = null;
let asking = false;

export function checkForUpdate() {
  if (!isTauri()) return Promise.resolve({ status: "offline" });
  if (!flight) {
    flight = invoke("update_check").finally(() => {
      flight = null;
    });
  }
  return flight;
}

export async function applyUpdate(report, messages, loud) {
  if (!report || report.status !== "available" || asking) return report;
  asking = true;
  try {
    const yes = await confirmAsk(String(messages.confirm || "").replace("{version}", report.latest || ""));
    if (!yes) return report;
    const installed = await invoke("update_install");
    if (installed && installed.status === "helper_missing" && installed.asset_url) {
      await invoke("open_url", { url: installed.asset_url });
    }
    return installed;
  } catch (err) {
    if (loud) throw err;
    return report;
  } finally {
    asking = false;
  }
}

export function launchUpdateCheck(messages) {
  checkForUpdate()
    .then((report) => applyUpdate(report, messages, false))
    .catch(() => {});
}

export function statusText(report, t) {
  const status = report && report.status;
  if (status === "current") return t["support.updateCurrent"];
  if (status === "available") return t["support.updateAvailable"].replace("{version}", report.latest || "");
  if (status === "no_package") return t["support.updateNone"];
  if (status === "rate_limited") return t["support.updateRate"];
  if (status === "bad_release") return t["support.updateBad"];
  if (status === "installed") return t["support.updateInstalled"];
  if (status === "helper_missing") return t["support.updateHelper"];
  return t["support.updateOffline"];
}
