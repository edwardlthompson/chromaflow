# ADR-0021: Native lighting without OpenRGB by default

- **Status:** Accepted
- **Date:** 2026-09-13
- **Deciders:** ChromaFlow lighting
- **Supersedes:** ADR-0013 spawn-by-default; ADR-0020 “OpenRGB stays oracle”
- **Related:** [ADR-0007](0007-mit-until-gpl-link.md), [ADR-0013](0013-bundled-openrgb-appimage.md)

## Context

OpenRGB on this host held GPU I2C and still returned SMBus `EIO` at `0x68`. Keychron, Arena, Prime, and Fusion already have native or sibling-CLI paths. Spawning the AppImage from the GUI and `chromaflow-sdk.service` kept bringing OpenRGB back.

## Decision

1. Do not spawn OpenRGB unless `CHROMAFLOW_OPENRGB_SDK=1`. `chromaflow daemon --sdk` exits 0 without a sibling. Session enablement **disables** `chromaflow-sdk.service`.
2. Lighting inventory is hidraw + liquidctl Fusion + PCI GPU row. No TCP 6742 probe when the SDK is off.
3. Apply: Keychron VIA, Arena/Prime HID, Fusion HID feature `0xCC` (liquidctl fallback). GPU lists this Suprim Liquid X (`10de:2684` / `1462:5104`) and writes ITE static RGB at I2C `0x68` on NVIDIA adapter 1 (no OpenRGB C++ map; never dump the bus).
4. AppImage may remain on disk as opt-in. Never paste Controllers/*.cpp. Do not kill `chromaflowd`.

## Alternatives considered

- Keep OpenRGB as silent fallback — rejected; it respawned and wedged GPU I2C
- Translate MSIGPUv2 registers from OpenRGB headers — rejected (ADR-0007)

### Critique

| Issue | Resolution |
|-------|------------|
| Null hidraw / liquidctl | Typed Err; Lighting lists only present backends |
| Network timeout | No 6742 probe when SDK off |
| Race GUI vs leftover AppImage | GUI does not `ensure_sdk`; unit disabled |
| Unhandled I2C | GPU apply Err if `0x68` missing; no speculative writes |
| GPL paste | liquidctl subprocess + on-device HID only |

## Consequences

Motherboard and USB RGB work without OpenRGB. GPU shroud RGB waits on a live I2C map. `CHROMAFLOW_OPENRGB_SDK=1` restores the sibling.
