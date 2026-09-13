# Feature: chromaflowd

> Cooling watchdog + `ExecStopPost` firmware failsafe (ADR-0018). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: `chromaflow daemon --watchdog` applies curves; unit `ExecStopPost` restores `pwm*_enable=2`
- ✅ Offline/error behavior: daemon without a mode flag exits 2; `--dry-run` never writes; failsafe never writes duty 0
- ✅ Accessibility: N/A (no GUI)
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ `CHROMAFLOW_DAEMON_ONCE=1` and a fake `CHROMAFLOW_HWMON_ROOT`
2. _When_ `chromaflow daemon --watchdog` runs
3. _Then_ owned `pwm*_enable` is 1 and duty follows the curve; failsafe restores 2

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/pwm_policy.rs` `pwm_apply.rs` `pwm_daemon.rs` |
| View | `packaging/chromaflowd.service` |
| Tests | `tests/test_chromaflow_daemon.py` |
| Wiring | `packaging/chromaflowd`, `packaging/pwm-failsafe.sh`, Cooling `pwm_takeover` |

## Tests

- Automated: yes — fake sysfs; policy is firmware enable (2) not 0
- Coverage: `--watchdog` once; `--dry-run` still no writes

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-SAFE. Live take-over still refuses while competing daemons are present.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
