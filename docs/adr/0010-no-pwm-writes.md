# ADR-0010: No PWM writes until watchdog

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Setting PWM to 0% can stop cooling. A GUI slider in a stub would be enough to damage hardware. The brief forbids silent 0% and requires `pwm*_enable` plus failsafe.

## Decision

Product crates **must not** expose `set_pwm` or write sysfs `pwm*` / `pwm*_enable`. The curve UI is display-only. Adding writes requires:

1. `chromaflowd` with a watchdog
2. Failsafe to 100% or kernel default on death (`ExecStopPost`)
3. Explicit user action for 0%
4. Conflict warning for CoolerControl / fancontrol / fan2go

CI greps for forbidden write APIs.

## Alternatives considered

- Read-write CLI behind `--i-know` — rejected; too easy to script accidentally

### Critique

| Issue | Resolution |
|-------|------------|
| Milestone cannot demo a live curve | Inventory + placeholder graph still prove the GUI shell |

## Consequences

Blocked change: PWM writes without this ADR being superseded.
