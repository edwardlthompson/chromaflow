# Feature: first-scheme

> Extra kernel install skips work already done, lists liquidctl, Cooling temp cards show 60 s graphs, chrome stays put, and `cooling --takeover` applies Balanced plus auto-calibrate. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Extra kernel support includes `liquidctl`; Install all / Install skip extras that are already present (no re-modprobe, no DKMS reload). Cooling temp cards show usage % plus full-width 60 s usage (green→red, 0–100%) and temp (blue→red, 30–90 °C) graphs. Top bar and left tab rail do not scroll. `chromaflow cooling --takeover` writes a Balanced scheme (min 20%, never silent 0%), enables `chromaflowd`, and sweeps tach headers.
- ✅ Offline/error behavior: unknown extra still rejected before pkexec; Vite cannot write PWM; conflicts skip motherboard PWM and still allow NVIDIA; calibrate skip when no `fanN_input`; failsafe remains `pwm*_enable=2`
- ✅ Accessibility: spark SVGs `role="img"`; conflict banner unchanged
- ✅ i18n: `lighting.extras.liquidctl` `cooling.usageGraph` `cooling.tempGraph`

## Smoke scenario

1. _Given_ Extra kernel support and Cooling on this machine
2. _When_ the user installs liquidctl and runs take-over
3. _Then_ liquidctl is present, chrome stays fixed while the main pane scrolls, temp cards fill 60 s graphs, and watchdog duty is above 0% on owned headers

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/lib/chromaflow_extras.py` `chromaflow_apply.py` `crates/chromaflow-core/src/pwm_scheme.rs` `apps/desktop/src/lib/spark.js` |
| View | `TempsList.svelte` `app.css` Extra kernel support |
| Tests | `tests/test_chromaflow_support.py` `tests/test_chromaflow_cooling_ui.py` rust `pwm_scheme` |
| Wiring | `install-support.sh --only liquidctl`; `chromaflow cooling --takeover`; Tauri `pwm_load` |

## Tests

- Automated: yes — extra id order, pending skip, spark 30–90 band, overflow hidden, scheme never `set_pwm`
- Coverage: liquidctl apt `--only`; Balanced min 20%; conflicts skip hwmon

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist). Live RPM is HUMAN smoke.
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-SAFE (ADR-0018). No CoolerControl apt. No bundled `.ko`. GUI/CLI never root.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
