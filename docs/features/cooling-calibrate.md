# Feature: cooling-calibrate

> Duty% ↔ RPM sweep after take-over. Never silent 0%. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Auto-calibrate on the Cooling toolbar takes over writable ITE fans then pumps, sweeps 20–100% duty, records RPM, restores the recipe tick; 100% still 0 RPM marks no tach and stores an empty table; NVIDIA GPU fans are skipped (watchdog applies them via `nvidia-settings` with `DISPLAY=:0`)
- ✅ Offline/error behavior: Vite shows native-window copy; conflicts or missing take-over refuse; abort restores `pwm*_enable=2`; never writes 0% unless `allow_zero`
- ✅ Accessibility: toolbar calibrate button labelled; progress in `role="status"`
- ✅ i18n: `cooling.calibrate` `cooling.calibrateProgress` `cooling.calibrateDone` `cooling.noTach` `cooling.needsNative` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ take-over is confirmed and no `conflicts[]`
2. _When_ the user clicks Auto-calibrate at the top right of Cooling
3. _Then_ each writable fan then pump steps 20, 30, … 100 (skip 0), RPM is stored or the table stays empty if tach is 0 at 100%, and the named curve tick resumes

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/pwm_calibrate.rs` `apps/desktop/src/lib/coolingSweep.js` |
| View | `apps/desktop/src/pages/Cooling.svelte` |
| Tests | `tests/test_chromaflow_daemon.py` rust fake-sysfs sweep |
| Wiring | Tauri `pwm_calibrate` (allow-pwm); `coolingSweep.js` queues fans then pumps |

## Tests

- Automated: yes — fake sysfs sweep never writes 0%; 0 RPM tach kept empty; no `set_pwm`
- Coverage: take-over required; pause file so chromaflowd does not tick mid-sweep

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Calibration is duty% ↔ RPM (Super I/O PWM), not a voltage rail. Live sweep stays HUMAN smoke.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
