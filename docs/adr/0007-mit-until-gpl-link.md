# ADR-0007: MIT until an in-process GPL link

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Bootstrap init supports MIT or Apache-2.0. The brief said GPL-3.0 if OpenRGB or Fan Control code is copied. Fan Control cannot be copied (proprietary). OpenRGB is GPL-2.0-or-later. liquidctl is GPL-3.0-or-later.

## Decision

Ship **MIT** + [`NOTICE`](../../NOTICE). Treat `openrgb` and `liquidctl` as optional **separate programs**. Do not vendor their trees. Stop and ask before linking OpenRGB C++ or adding a GPL crates.io SDK client.

## Alternatives considered

- GPL-3.0 from day one — possible, but bootstrap license flag cannot emit GPL and we copy no GPL sources yet
- Apache-2.0 — no advantage over MIT here

### Critique

| Issue | Resolution |
|-------|------------|
| Calling GPL binaries from MIT | Documented as separate works in NOTICE |
| Accidental paste of NetworkClient.cpp | This milestone only TCP-connects localhost; no protocol dump from OpenRGB sources |

## Consequences

`[HUMAN]` must approve any copyleft crate. Relicense ADR required before in-process OpenRGB.
