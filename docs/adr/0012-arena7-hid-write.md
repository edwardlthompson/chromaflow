# ADR-0012: Arena 7 vendor HID color writes

- **Status:** Accepted (Prime Neo hidraw is [ADR-0014](0014-prime-neo-hid-write.md))
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow lighting follow-up
- **Related:** ADR-0011 (OpenRGB SDK / liquidctl Fusion)

## Context

OpenRGB git2478 does not list SteelSeries Arena 7 (`1038:1a00`). The device is USB audio plus two hidraw interfaces; only interface 4 is vendor page `0xFFC0` with output report ID `0x06`. Motherboard/keyboard/GPU color apply already works. Chassis radiator ARGB is Fusion `D_LED1`/`D_LED2`, not the GPU I2C logo.

## Decision

Unprivileged code **may write** a 64-byte **output** report to the Arena 7 vendor hidraw only (`HID_ID` `1038:1a00` and descriptor prefix `06 c0 ff`). Layout: `[0x06][0xA1][4× zone][0x0F][pad]`, zone = `[R,G,B,0x01,0x1E,10]`. This matches the public Prismatic-for-macOS protocol notes (MIT). No SteelSeries GG. No PWM. Prime Neo is ADR-0014.

Fusion apply also sends OpenRGB `RESIZEZONE` (1000) to 32 LEDs and `UPDATE_ZONE_LEDS` (1051) for zones 0–1 (`D_LED1`/`D_LED2`) so addressable radiator headers are not left at size 0.

## Alternatives considered

- OpenRGB Arena backend — not present on git2478; do not vendor C++
- SteelSeries GG / Windows VM — not the Linux product path
- hidraw for Prime Neo — [ADR-0014](0014-prime-neo-hid-write.md) (rivalcfg-documented 0x62 / 0x59)

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty color | Same RRGGBB parse as ADR-0011; tests cover report layout |
| Network timeout | N/A — local hidraw write; OpenRGB D_LED uses existing 2s I/O |
| Race | Vendor hidraw is not the Fusion node OpenRGB holds; SDK mutex still serializes D_LED |
| Unhandled exceptions | `Result` from write; UI/CLI show the error |
| Wrong hidraw | Skip consumer-control hidraw (`hidraw9`); require `0xFFC0` descriptor |
| GPL / proprietary SDK | Packet bytes from public MIT notes; no GG, no OpenRGB controller sources |

## Consequences

ADR-0011’s “no hidraw SET_REPORT” is superseded for Arena 7 output report `0x06`. Prime Neo wheel LED is ADR-0014.
