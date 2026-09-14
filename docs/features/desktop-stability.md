# Feature: desktop-stability

> Reap OpenRGB when it dies; do not call nvidia-settings when GPU fan percent is unchanged. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: if the OpenRGB AppImage exits, `chromaflow daemon --sdk` reaps it and starts it again (30s backoff after a failed spawn); Cinnamon is not hit with `nvidia-settings -a` every watchdog tick while Quiet/Balanced percent is already applied
- ✅ Offline/error behavior: missing binary is still a no-op; 0% GPU duty still needs confirm; PWM watchdog is never killed to recover lighting
- ✅ Accessibility: N/A (daemons)
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ OpenRGB aborting or a zombie AppImage child
2. _When_ `ensure_sdk` runs
3. _Then_ `/proc/pid/stat` state `Z` is not treated as live, the `Child` is `try_wait`ed, and a new server is spawned only if X is up and the 30s hold has expired
4. _Given_ GPU fans already at 20%
5. _When_ the watchdog ticks again
6. _Then_ `nvidia-settings -a GPUTargetFanSpeed` is not run

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/openrgb_spawn.rs` `nvidia_fan_apply.rs` `nvidia_fans.rs` |
| Packaging | `scripts/build-chromaflow-deb.sh` (`libxcb-cursor0`) |
| Tests | `tests/test_chromaflow_daemon.py` `tests/test_chromaflow_packaging.py` crate unit tests |

## Tests

- Automated: yes — no `mem::forget`; `proc_state` zombie vs sleeping; skip-repeat GPU percent; `libxcb-cursor0` in Depends; no `set_pwm`
- Coverage: 0% still refused; failsafe clears last GPU percent

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
- Live: `systemctl --user restart chromaflow-sdk` after X is up (never `chromaflowd`)

## Definition of Done

OpenRGB is not left as a zombie. GPU fan apply is idempotent. G-SAFE unchanged.
