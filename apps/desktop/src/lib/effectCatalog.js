/** Host effect menus. All Devices is the intersection every backend can run. */

export const EFFECT_GROUPS = [
  { id: "solid", modes: ["Solid Color", "Breathing", "Flashing"] },
  {
    id: "rainbow",
    modes: [
      "Cycle All",
      "Cycle Left Right",
      "Cycle Up Down",
      "Rainbow Moving Chevron",
      "Cycle Out In",
      "Cycle Out In Dual",
      "Cycle Pinwheel",
      "Cycle Spiral",
      "Band Spiral",
      "Rainbow Beacon",
    ],
  },
  { id: "motion", modes: ["Jellybean Raindrops", "Pixel Rain", "Splash"] },
  { id: "hardware", modes: ["Hardware gauges", "CPU", "GPU", "Combined", "RAM", "Disk"] },
];

export const HOST_EFFECTS = EFFECT_GROUPS.flatMap((g) => g.modes);

export const SHARED_MODES = ["Solid Color", "Breathing", "Cycle All", "Combined"];

const UNIFORM_MODES = [
  "Solid Color",
  "Breathing",
  "Flashing",
  "Cycle All",
  "Hardware gauges",
  "CPU",
  "GPU",
  "Combined",
  "RAM",
  "Disk",
];

export function groupedModes(names) {
  const want = new Set(names || []);
  return EFFECT_GROUPS.map((g) => ({ id: g.id, modes: g.modes.filter((m) => want.has(m)) })).filter(
    (g) => g.modes.length,
  );
}

export function deviceModes(device) {
  const b = (device && device.backend) || "";
  if (b === "liquidctl" || b === "prime" || b === "msi_gpu") return UNIFORM_MODES.slice();
  return HOST_EFFECTS.slice();
}
