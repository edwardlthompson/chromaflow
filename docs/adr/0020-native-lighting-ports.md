# ADR-0020: Native lighting ports with OpenRGB identity catalog

- **Status:** Accepted
- **Date:** 2026-09-13
- **Deciders:** ChromaFlow native lighting
- **Related:** [ADR-0007](0007-mit-until-gpl-link.md), [ADR-0013](0013-bundled-openrgb-appimage.md), [ADR-0015](0015-keychron-led-readback.md)

## Context

OpenRGB remains the sibling on `127.0.0.1:6742`. MIT forbids translating `Controllers/*.cpp`. The product still needs a complete **gap list** of devices OpenRGB 1.0 already supports, and a native HID path on this host so USB color does not depend on Qt/TCP.

## Decision

1. Keep MIT. Relicense is not chosen. OpenRGB AppImage stays spawned as oracle/fallback (`CHROMAFLOW_NATIVE_RGB=0` forces SDK-only).
2. Pin OpenRGB git revision next to the AppImage in `data/openrgb-engine.yaml`. `scripts/fetch-openrgb-device-index.sh` shallow-clones that revision (never vendored) and writes **identity only** to `data/openrgb-device-index.csv` (name, bus, VID/PID/SVID/SPID, detector function). Overlay `data/native-lighting.yaml` marks `sdk` / `native` / `extra`.
3. `lighting_port` claims Arena, Prime, and Keychron VID `3434`. Apply prefers native, then OpenRGB. List Keychron from hidraw even when SDK names are thin.
4. Keychron Direct is native HID. Uniform picker fills set rgb_matrix SOLID (`0x01`) + HSV. Host effect frames (including Cycle All) stay in Direct SET_COLOR even when every key shares a hue, so the clock can advance. Arena host frames write 4 HID zones. Motherboard Fusion is onboard analog (`led1`,`led3`,`led4`). CPU AIO is header analog plus a D_LED HID fill. Cycle All is one host hue on every lamp, started in the same tick; Breathing/Flash on Fusion stay firmware. Do not paste OpenRGB C++. GPU shroud writes ITE static RGB at I2C `0x68` on NVIDIA adapter 1 (20 ms pacing; never dump the bus).

## Alternatives considered

- GPL product + vendor OpenRGB — rejected (ADR-0007)
- Markdown dump of every OpenRGB device in `PRODUCT_GAPS.md` — rejected; CSV + counts
- Copy `Supported Devices.csv` from OpenRGB CI — rejected; we extract identity ourselves

### Critique

| Issue | Resolution |
|-------|------------|
| Null hidraw / wrong interface | VIA descriptor `06 60 ff` only; missing node is typed Err |
| Network timeout | Native HID has none; SDK compare stays 200ms optional |
| Race native vs SDK on Keychron | Native apply `drop_session`; skip SDK `UPDATE_LEDS` for claimed VID `3434` |
| Unhandled USB errors | Apply returns `Err`; Lighting `role="alert"` |
| GPL paste | Identity CSV/class names only; SET verified by GET_COLOR on this Q6 HE |

## Consequences

G-LED-NATIVE is the OpenRGB-wide backlog. This host is the lab. Windows stays Rust+Tauri with later `cfg(windows)` HID ports. Do not kill `chromaflowd`.
