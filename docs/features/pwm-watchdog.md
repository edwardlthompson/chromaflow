# Feature: pwm-watchdog

> Watchdog + firmware failsafe so Cooling can apply duty (ADR-0018). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling take-over is a confirm, then the watchdog writes duty and enables `chromaflowd`; header says the daemon keeps duty after the window closes. 0% needs a second confirm. Conflicts keep take-over disabled.
- ✅ Offline/error behavior: unwritable `pwm*` or Vite preview does not write sysfs; missing sensor restores that channel to firmware enable 2
- ✅ Accessibility: take-over checkbox labelled; failsafe status in `role="status"`
- ✅ i18n: `cooling.takeoverAsk` `cooling.zeroAsk` `app.pwmOn` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ no competing fan daemon and writable `pwm*` (or a fake `CHROMAFLOW_HWMON_ROOT`)
2. _When_ the user confirms take-over (or `chromaflow daemon --watchdog` with `CHROMAFLOW_DAEMON_ONCE=1`)
3. _Then_ `pwm*_enable` is 1 and duty follows the curve; on stop, failsafe writes enable 2, never 0

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/pwm_policy.rs` `crates/chromaflow-core/src/pwm_apply.rs` |
| View | `apps/desktop/src/pages/Cooling.svelte` `apps/desktop/src/lib/pwm.js` |
| Tests | `tests/test_chromaflow_daemon.py` `tests/test_chromaflow_cooling_ui.py` |
| Wiring | `crates/chromaflow-cli/src/daemon.rs` `packaging/pwm-failsafe.sh` Tauri `pwm_tick` |

## Tests

- Automated: yes — fake sysfs apply + failsafe; daemon `--watchdog` once; no `set_pwm` symbol
- Coverage: 0% refused; enable never 0; owned-only restore

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-SAFE: watchdog + `ExecStopPost` failsafe; Cooling can apply when writable and unconflicted. GUI close skips failsafe while `chromaflowd` is active.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
