# ADR-0015: Keychron per-LED HID readback at display refresh

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow lighting follow-up
- **Related:** ADR-0011 (OpenRGB SDK writes)

## Context

OpenRGB `REQUEST_CONTROLLER_DATA` and Keychron firmware Rainbow Wave do not share a live LED framebuffer. SDK `led_colors` stay on the last Direct buffer. The Q6 HE VIA interface (`hidraw` usage page `0xFF60` usage `0x61`, VID `3434`) answers Keychron RGB command `0xA8` subcommand `0x09` (GET_COLOR) with up to nine HSV triples per 32-byte report. That is a real per-key read, not a UI guess. Firmware matrix effects still do not dump the animation RAM; GET_COLOR is the per-key buffer. While the LED layout is open, ChromaFlow therefore switches that keyboard to Direct, writes a host frame, then paints the HID readback.

## Decision

- Poll every Keychron LED via GET_COLOR (`0xA8`/`0x09`) on the VIA hidraw only (descriptor prefix `06 60 ff`). Convert HSV bytes to `#rrggbb`. A background thread kicks as fast as USB allows; the GUI uses `requestAnimationFrame` so paint matches the monitor (including 144 Hz).
- `lighting_sync` may `SET_CUSTOM` + `UPDATE_LEDS` with per-LED RGB, then overlay HID colors on the Keychron row. No PWM. No OpenRGB C++.

## Alternatives considered

- Trust SDK DATA during firmware effects — rejected; this host returned a solid leftover Direct color while the keys ran Rainbow Wave.
- Simulate the named effect in the layout — rejected; the pattern did not match the device.
- One HID GET per LED at 144 Hz — USB interrupt cannot carry 108 round-trips per frame; batches of 9 are the packet limit.

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty hidraw | Missing VIA node returns no overlay; SDK pixels still show |
| Network timeout | Localhost SDK 80 ms; HID is a char device on a worker thread |
| Race with OpenRGB | `lock_sdk` around TCP; apply drops both SDK and HID sessions |
| Unhandled exceptions | Worker failures keep the last good snapshot |
| GPL | Command bytes probed on this Q6 HE; no copied OpenRGB sources |

## Consequences

Expanding the Keychron LED card drives Direct so readback matches the keys. Collapsed, firmware modes can keep running without that stream. PWM unchanged.
