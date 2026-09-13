# Feature: cooling-ui

> Fan Control-like Fans, Pumps, Temps, and Curves with confirmed PWM take-over. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling has Fans / Pumps / Temps / Curves; one Auto-calibrate in the Cooling toolbar sweeps every writable fan then pump (20–100% duty↔RPM, never silent 0%); fan/pump cards omit Tune and per-card calibrate; every Cooling board tile is `22rem` (fan/pump/temp/curve); curve graphs fill the leftover card height with °C/% labels in the SVG; Lighting/Profiles/Support unmount off-tab; Cooling inventory is hwmon-only (no OpenRGB spawn); 60 s graphs have a 0–100% / 30–90 °C wireframe (vertical bar every 15 s); **+** next to Temps adds a mix; **+** / **Clone** next to Curves copies a preset into an editable card (Quiet/Balanced/Performance stay locked); lighting USB/OpenRGB ticks run only on Lighting; inventory poll pauses while Cooling is scrolling; **Apply to all** persists in `curves.json` across launches (hydrate waits for this-machine inventory, not the sample fixture)
- ✅ Offline/error behavior: invalid temps flagged; empty hwmon shows “No hwmon chips”; Vite cannot write sysfs; 0 RPM headers are hidden (Hidden row; restore shows the card)
- ✅ Accessibility: conflict banner `role="alert"`; curve `role="img"`; mix/curve plus buttons have `aria-label`
- ✅ i18n: keys under `cooling.*` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ `chromaflow-gui` is running on this machine
2. _When_ the user opens Cooling and scrolls the fan board
3. _Then_ they see named ITE headers plus NVIDIA GPU fans, every board card the same 22rem height as a fan card, one Auto-calibrate at the top right, Temps + and Curves +, and no mix checkboxes until + is clicked
4. _When_ they click Quiet **Apply to all**, quit the GUI, and launch again
5. _Then_ on-board fan/pump dropdowns still say Quiet and `~/.config/chromaflow/curves.json` still has those channels `enabled` with `curve_id` quiet

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/lib/cooling.js` `coolingBoard.js` `coolingMix.js` `coolingCurves.js` `coolingSweep.js` `spark.js` `crates/chromaflow-core/src/pwm_recipe.rs` |
| View | `apps/desktop/src/pages/Cooling.svelte` `TempsList.svelte` `SparkGraph.svelte` `CurvesList.svelte` |
| Tests | `tests/test_chromaflow_cooling_ui.py` plus daemon fake-sysfs |
| Wiring | `App.svelte` unmounts idle tabs; Cooling `inventory` `{ light: false }` → `collect_cooling`; Tauri `pwm_takeover` schema 2; custom curves in `curves.json` |

## Tests

- Automated: yes — `tests/test_chromaflow_cooling_ui.py` plus three-channel daemon fake-sysfs
- Coverage: milliC, presets, pickList, custom `points_for`, mix Max CPU+GPU, no GUI `pwm_tick`, no `set_pwm`

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-UX / G-COOL / G-SAFE (ADR-0018). File `.sensor` mixes are skipped (PRODUCT_GAPS).

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
