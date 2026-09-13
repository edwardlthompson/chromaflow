/** HID leftover after OpenRGB, liquidctl, and known VID:PID backends. */

import { hidList, liquidctlList, sdkTargets, arenaTargets, primeTargets, fusionFallback } from "./lighting.js";

function sdkBlob(inventory) {
  return ((inventory && inventory.openrgb && inventory.openrgb.controllers) || [])
    .map((r) => String((r && r.name) || "").toLowerCase())
    .join(" ");
}

function liquidBlob(inventory) {
  return liquidctlList(inventory).join(" ").toLowerCase();
}

function noiseHid(d) {
  const n = `${(d && d.name) || ""} ${(d && d.hid_name) || ""}`.toLowerCase();
  return ["ups", "microphone", "fingerprint", "camera", "touchpad", "wacom", "cs201", "consumer control"].some(
    (k) => n.includes(k),
  );
}

export function hidHasBackend(d, inventory) {
  const vid = String((d && d.vendor_id) || "").toLowerCase();
  const pid = String((d && d.product_id) || "").toLowerCase();
  if (vid === "1038" && (pid === "1a00" || pid === "1856")) return true;
  const sdk = sdkBlob(inventory);
  const liquid = liquidBlob(inventory);
  if (vid === "048d" && pid === "5702") {
    return /aorus|gigabyte|fusion/.test(sdk) || liquid.includes("fusion");
  }
  if (vid === "3434" && sdk.includes("keychron")) return true;
  const name = String((d && d.name) || "").toLowerCase();
  if (!name) return false;
  return sdk.includes(name) || controlledNames(inventory).includes(name);
}

function controlledNames(inventory) {
  return sdkTargets(inventory)
    .concat(arenaTargets(inventory))
    .concat(primeTargets(inventory))
    .concat(fusionFallback(inventory))
    .map((d) => String((d && d.name) || "").toLowerCase());
}

export function fusionUsbListed(inventory) {
  if (liquidBlob(inventory).includes("fusion")) return true;
  return hidList(inventory).some((d) => d.vendor_id === "048d" && d.product_id === "5702");
}

export function researchDevices(inventory) {
  const out = [];
  for (const d of hidList(inventory)) {
    if (noiseHid(d) || hidHasBackend(d, inventory)) continue;
    let reason = "unknown_usb";
    if (!d.readable) reason = "hidraw_blocked";
    else reason = "not_in_openrgb";
    out.push({ ...d, reason });
  }
  for (const name of liquidctlList(inventory)) {
    if (/fusion/i.test(name) || sdkBlob(inventory).includes(name.toLowerCase())) continue;
    out.push({
      name,
      reason: "not_in_openrgb",
      vendor_id: "",
      product_id: "",
      readable: true,
      kind: "other",
      path: "",
      hid_name: name,
    });
  }
  return out;
}
