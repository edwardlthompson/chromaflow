# ADR-0008: OpenRGB localhost probe only

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Lighting should reuse OpenRGB backends rather than re-reverse-engineer protocols. The SDK is an unauthenticated TCP protocol. Copying OpenRGB client C++ would pull GPL into the tree.

## Decision

This milestone **connects to `127.0.0.1:6742` with a timeout** and reports reachable vs unreachable. No mode/color packets. Do not spawn `openrgb --server`. Do not use Wine or Windows plugins.

If we later spawn a server, bind localhost only.

## Alternatives considered

- Vendor OpenRGB as a submodule — license + build cost; deferred
- Third-party crates.io OpenRGB client — check license first; not used yet

### Critique

| Issue | Resolution |
|-------|------------|
| Empty device list | UI shows server missing vs no Linux backend; no fake LEDs |

## Consequences

Device names from OpenRGB appear only after a later SDK read ADR.
