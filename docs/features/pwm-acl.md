# Feature: pwm-acl

> Plugdev PWM sysfs ACL plus Cooling enable of `chromaflowd` so duty survives GUI close. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Support extra `pwm_acl` installs `pwm-acl.sh`; Cooling take-over runs `systemctl --user enable --now chromaflowd`; GUI close does not failsafe while the daemon is active
- ✅ Offline/error behavior: missing `pwm-acl.sh` is an extra error; Vite cannot write sysfs; enable failure is shown in the take-over detail
- ✅ Accessibility: existing take-over switch and Support extra row
- ✅ i18n: `lighting.extras.pwm_acl` `cooling.takeoverAsk` `app.pwmOn`

## Smoke scenario

1. _Given_ the helper `.deb` is installed and no competing fan daemon
2. _When_ the user installs `pwm_acl` and confirms Cooling take-over
3. _Then_ `pwm*` is group-writable by plugdev, `chromaflowd` is enabled, and closing the GUI leaves duty under the daemon

## Container map

| Layer | Path |
|-------|------|
| Logic | `packaging/pwm-acl.sh` `crates/chromaflow-core/src/pwm_daemon.rs` `scripts/lib/chromaflow_extras.py` |
| View | Support extra list; Cooling take-over copy |
| Tests | `tests/test_chromaflow_daemon.py` `tests/test_chromaflow_lighting.py` |
| Wiring | udev `RUN+` `pwm-acl.sh`; Tauri `pwm_takeover` / `pwm_release` |

## Tests

- Automated: yes — fake sysfs chmod without duty write; extras id; udev line; `.deb` path; no `set_pwm`
- Coverage: ACL never writes duty; daemon enable is `--user --now`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-SAFE: plugdev can write PWM; `chromaflowd` outlives the GUI; failsafe still restores firmware when the daemon is not active.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
