# Feature: cooling-fans

> Detect every cooling control on this host: ITE headers, NVIDIA hybrid GPU fans, AIO unit grouping. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling lists Gigabyte ITE headers (`CPU_FAN` / `SYS_FAN*` / `CPU_OPT` on IT8689, `FAN4`–`FAN8` on IT87952 including ENODATA `pwm4`/`pwm5` that become writable after `pwm*_enable=1`) plus NVIDIA `GPU fan 0/1`; pump headers default to Pumps; AIO unit take-over applies one recipe to GPU fans + pump headers
- ✅ Offline/error behavior: 0 RPM headers stay listed; NVIDIA apply never writes 0% without confirm; missing primary IT8689 is `ite_primary_missing` (no shipped `.ko`); `chromaflowd` sets `DISPLAY=:0` so GPU apply works without a GUI terminal
- ✅ Accessibility: existing Cooling labels; AIO unit checkbox labelled
- ✅ i18n: `cooling.aioUnit` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ `chromaflow-gui` on this Gigabyte X570S + RTX 4090 hybrid
2. _When_ the user opens Cooling
3. _Then_ they see every writable ITE PWM (including IT87952 `pwm4`/`pwm5`) plus NVIDIA fans, pump headers grouped as an AIO unit with the GPU fans, and empty headers stay listed at 0 RPM

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/nvidia_fans.rs` `nvidia_fan_apply.rs` `gaps.rs` `apps/desktop/src/lib/cooling.js` `coolingBoard.js` |
| View | `apps/desktop/src/pages/Cooling.svelte` `ControlList.svelte` |
| Tests | `tests/test_chromaflow_cooling_ui.py` plus rust `nvidia_fans` / `gaps` tests |
| Wiring | `scan.rs` `gpu_fans`; Tauri `pwm_takeover` already ticks `pwm_apply` which calls NVIDIA apply |

## Tests

- Automated: yes — parse two `nvidia-settings` fans; X570S IT87952-only is `ite_primary_missing`; UI strings for AIO unit and FAN4/FAN7; hwmon lists unread `pwm4`
- Coverage: never silent 0% NVIDIA; no `set_pwm`

## Fallback validation

- Why tests are not feasible: N/A (automated tests exist)
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-COOL detection of GPU fans + named ITE headers + AIO unit. IT8689 PWM is Support extra `it87-dkms` (not a ChromaFlow `.ko`).

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
