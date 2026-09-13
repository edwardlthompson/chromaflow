# ADR-0008: OpenRGB localhost probe only

- **Status:** Superseded by [ADR-0011](0011-openrgb-sdk-write.md) for color/mode packets and [ADR-0013](0013-bundled-openrgb-appimage.md) for sibling spawn; localhost-only probe still applies
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Lighting should reuse OpenRGB backends rather than re-reverse-engineer protocols. The SDK is an unauthenticated TCP protocol. Copying OpenRGB client C++ would pull GPL into the tree.

## Decision

This milestone **connects to `127.0.0.1:6742` with a timeout** and reports reachable vs unreachable. No mode/color packets in this ADR. Do not use Wine or Windows plugins.

Spawning a localhost server is [ADR-0013](0013-bundled-openrgb-appimage.md) (sibling engine, not in-process).

## Alternatives considered

- Vendor OpenRGB as a submodule — license + build cost; deferred
- Third-party crates.io OpenRGB client — check license first; not used yet

### Critique

| Issue | Resolution |
|-------|------------|
| Empty device list | UI shows server missing vs no Linux backend; no fake LEDs |
## Consequences

Device names from OpenRGB appear after the SDK list probe. Color packets are [ADR-0011](0011-openrgb-sdk-write.md).
