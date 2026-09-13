# ADR-0014: Prime Neo wheel LED hidraw

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow lighting follow-up

## Context

OpenRGB does not list SteelSeries Prime Neo (`1038:1856`). SteelSeries GG on Windows can set the scroll-wheel LED. [Rivalcfg](https://flozz.github.io/rivalcfg/devices/prime.html) documents the same PID with CLI `--color` on Linux. We do not vendor rivalcfg (GPL-3).

This host exposes four hidraw nodes for that VID:PID. Interface with HID descriptor prefix `06 c0 ff` (`0xFFC0`) is the vendor page, same class as Arena 7.

## Decision

Unprivileged code **may write** HID **output** reports to that vendor hidraw only:

- Color: report ID `0x00`, command `0x62 0x01`, RGB, 15 zero bytes, `0xFF`
- Save: `0x00 0x59` after 50ms

Command bytes match the public rivalcfg Prime device profile; the write is our hidraw path, not a rivalcfg spawn. No SteelSeries GG. No PWM.

## Alternatives considered

- OpenRGB SteelSeries mouse controller — PID 1856 is not registered
- `rivalcfg` subprocess — extra PATH tool; hidraw is already plugdev-readable
- Feature reports — rivalcfg uses HID output (`0x02`) for this color command

### Critique

| Issue | Resolution |
|-------|------------|
| Wrong hidraw | Require `1038:1856` and descriptor `06 c0 ff`; skip mouse/keyboard nodes |
| Null color | Same RRGGBB parse as ADR-0011; unit test report layout |
| Busy device | 50ms before save; `Result` on write |
| GPL | Cite public command bytes; no rivalcfg source in `crates/` |

## Consequences

Prime Neo appears on Lighting with Apply. Research list omits it when hidraw is readable. [ADR-0012](0012-arena7-hid-write.md) “Prime Neo display-only” is superseded for this packet only.
