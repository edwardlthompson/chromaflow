# ADR-0011: Localhost OpenRGB SDK color writes

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 8
- **Supersedes:** ADR-0008 “no mode/color packets”

## Context

USB hidraw is now readable after per-VID udev. Native OpenRGB on `127.0.0.1:6742` lists this board’s Fusion controller and other SDK devices. Lighting still must not vendor OpenRGB C++ or write PWM.

## Decision

Unprivileged code **may send** documented localhost SDK packets: `SET_CLIENT_NAME` (50), `SET_CUSTOM_MODE` (1100), `UPDATE_LEDS` (1050), `UPDATE_ZONE_LEDS` (1051), `UPDATE_SINGLE_LED` (1052), `UPDATE_MODE` (1101). `UPDATE_LEDS` / `UPDATE_MODE` `data_size` is the **full body**, equal to header `pkt_size`. Connect timeout stays **200ms**. Device index scheme (protocol 0 request body). Fusion also gets a `liquidctl` subprocess (`-m Fusion set sync color fixed RRGGBB`) even when the SDK lists the Aorus board, because this 5702 often ignores SDK LED buffers. Never a copy of `rgb_fusion2.py`.

Lighting offers color, mode, and per-LED apply for OpenRGB backends that exist (SDK names, Arena 7 vendor HID color-only, plus liquidctl Fusion). Effects are the modes each controller reports — not a hardcoded global enum. Prime Neo still has no Linux backend. No GUI as root. No `set_pwm`.

## Alternatives considered

- crates.io OpenRGB client — stop-and-ask (likely copyleft); not used
- Flatpak OpenRGB — sandboxed; native AppImage/`openrgb` only
- Direct hidraw for Arena 7 — ADR-0012 (Prime Neo still has no documented report)

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty color or device | Reject unless RRGGBB and a non-empty name; tests cover both |
| Network timeout | 200ms localhost; failure is a JSON/UI error, no retry storm |
| Race | Apply uses a fresh TCP session; UI rescan is already 2.5s |
| Unhandled exceptions | `Result`/`invoke` error; never silent success |
| GPL contamination | Packet IDs from the public OpenRGB SDK wiki only |

## Consequences

ADR-0008 remains the localhost-only probe rule. Color packets are this ADR. PWM stays ADR-0010.
