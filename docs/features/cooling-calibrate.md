# Feature: cooling-calibrate

> Duty% ↔ RPM sweep after take-over. Never silent 0%. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Auto-calibrate takes over writable ITE fans then pumps; **one header at a time** (peers at min) while that channel sweeps **20–100% duty in 10% steps** (~2s dwell); stores duty↔RPM in the recipe; restores the curve tick; 100% still 0 RPM marks no tach; NVIDIA skipped; cards do **not** show calibrated RPM text (Hide stays top-right)
- ✅ Identify: per-card **Identify** spins that header at 100% and peers at min for ~5s, then restores the recipe and offers a rename prompt (board labels alone cannot name “front intake”)
- ✅ Offline/error behavior: Vite shows native-window copy; conflicts or missing take-over refuse; per-header errors continue the board sweep; abort restores `pwm*_enable=2`; never writes 0% unless `allow_zero`; Windows `chip.path` uses `\` (JS `channelOf` matches Rust `dir_of`)
- ✅ Accessibility: toolbar calibrate + Identify labelled; progress in `role="status"`
- ✅ i18n: `cooling.calibrate*` `cooling.identify*` `cooling.noTach` `cooling.needsNative` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ take-over is confirmed and no `conflicts[]`
2. _When_ the user clicks Auto-calibrate at the top right of Cooling
3. _Then_ each writable fan then pump is solo’d (peers at min) and steps 20, 30, … 100 (skip 0); progress shows name n/total; recipe stores the table; curve tick resumes
4. _When_ the user clicks Identify on one card
5. _Then_ that header runs at 100% briefly, peers stay at minimum, recipe restores, and a rename prompt can set a friendly name

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/pwm_calibrate.rs` `apps/desktop/src/lib/coolingSweep.js` `apps/desktop/src/lib/coolingCal.js` |
| View | `apps/desktop/src/pages/Cooling.svelte` `apps/desktop/src/lib/ControlList.svelte` |
| Tests | rust fake-sysfs sweep + identify; `coolingCal.js` range helper |
| Wiring | Tauri `pwm_calibrate` `pwm_identify` (allow-pwm) |
## Tests

- Automated: yes — fake sysfs sweep never writes 0%; identify floors peers; 0 RPM tach kept empty; no `set_pwm`
- Coverage: take-over required; pause file so chromaflowd does not tick mid-sweep/identify

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Calibration is duty% ↔ RPM (Super I/O PWM), not a voltage rail. Identify is listen/look + rename — headers cannot auto-fingerprint a consumer product. Live sweep stays HUMAN smoke.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
