# ADR-0018: PWM watchdog and firmware failsafe

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow PWM take-over
- **Supersedes:** [ADR-0010](0010-no-pwm-writes.md)

## Context

ADR-0010 blocked all `pwm*` writes until a watchdog and failsafe existed. Silent 0% can stop cooling. Competing daemons still must not be fought. The GUI and `chromaflow` CLI stay unprivileged.

## Decision

PWM duty is written only by `chromaflow daemon --watchdog` or the same `pwm_apply` tick the GUI invokes after an explicit take-over confirm:

1. `pwm*_enable` is **1** (manual) while ChromaFlow owns a channel; never **0**.
2. Duty **0%** requires a second confirm (`allow_zero`). Default minimum is 20%.
3. Conflicts (`fancontrol`, `coolercontrold`, `fan2go`, …) refuse take-over.
4. On daemon stop, GUI close, or a missing sensor, failsafe writes **`pwm*_enable=2`** (motherboard firmware) for owned channels only — never duty 0.
5. `ExecStopPost=/usr/libexec/chromaflow/pwm-failsafe.sh` is the systemd hook. Rust `failsafe_at` is the in-process hook.
6. No `set_pwm` / `write_pwm` symbols. Paths must be `hwmonN/pwmN` under `CHROMAFLOW_HWMON_ROOT`.

## Alternatives considered

- Privileged `chromaflow` CLI — rejected; `refuse_if_root` stays. Writable sysfs is udev/`plugdev`.
- Failsafe to 100% duty — rejected; firmware enable (2) is the kernel default on this Super I/O.

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty owned list | Failsafe is a no-op; test empty owned file |
| Write timeout | Sysfs write is local; tick errors restore that channel |
| Race: GUI tick vs daemon | Same curves file; last apply wins; failsafe still enable=2 |
| Unhandled EACCES | Tick returns error; Cooling shows it; no retry storm |
| Silent 0% | `percent_to_duty` Err unless `allow_zero`; test |
| Conflicts | `tick` refuses; Cooling checkbox stays disabled |

## Consequences

Cooling take-over is live when PWM is writable and no competitor is present. Vite preview cannot write sysfs. CI uses a fake hwmon tree.
