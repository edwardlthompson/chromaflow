# ADR-0016: Host Direct effects for every OpenRGB device

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow lighting follow-up
- **Related:** ADR-0011 (SDK writes), ADR-0015 (Keychron HID readback)

## Context

Firmware matrix effects do not dump animation RAM. OpenRGB `led_colors` stay on the last Direct buffer. VIA GET_COLOR is the per-key store, not the running Rainbow Wave. A UI-only guess did not match QMK/OpenRGB math. Users need the same named effects on every controller (Keychron, Aorus, GPU AIO), including devices that never listed that firmware mode.

## Decision

ChromaFlow renders QMK RGB-matrix-style effects on the host (`effectQmk.js`) and pushes them with `SET_CUSTOM` + `UPDATE_LEDS` via `lighting_sync` to **every** OpenRGB controller. LED coordinates map from the SDK matrix (or index along 0–224). Apply effect does **not** send `UPDATE_MODE`. Apply color sets Direct and stops the engine. Keychron GET_COLOR overlay is skipped while host frames are in flight. Arena/Prime/liquidctl get the same catalog at ~30 Hz via color apply (LED 0). No OpenRGB C++.

## Alternatives considered

- Firmware `UPDATE_MODE` per device — rejected; skipped devices, white Direct flash, no per-LED truth
- Keychron-only Direct stream — rejected; Aorus and AIO need the same path
- Trust HID GET_COLOR during firmware — rejected; it is the stored color, not animation RAM

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty device or color | Skip empty frames; picker hex or last color; tests cover classify + frame |
| Network timeout | Preview pull is 400 ms; host-effect push skips DATA and returns written pixels |
| Race with firmware | Pull marks non-Direct modes unpainted so the next push re-sends `SET_CUSTOM` |
| Unhandled exceptions | Invoke errors stay in status; rAF continues |
| GPL | Compact JS from published QMK formulas; no vendored C++ |

## Consequences

The effect list is `HOST_EFFECTS`, not each controller’s firmware `modes[]`. Layout pixels are the colors just written. PWM unchanged.
