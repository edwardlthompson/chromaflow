# ADR-0009: .deb helper vs AppImage

- **Status:** Superseded (GUI AppImage). Helper pin still applies. Product install is [ADR-0013](0013-bundled-openrgb-appimage.md) sibling engine + `chromaflow_*.deb`.
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Polkit must execute a **pinned, root-owned** path. An AppImage extracted under `$HOME` is user-writable and must not be that path.

## Decision

- Helper install path: `/usr/libexec/chromaflow/install-support.sh`
- Product install unit is `chromaflow_0.1.0_amd64.deb` (GUI, CLI, `.desktop`, helper). ChromaFlow itself is **not** an AppImage.
- A later GUI AppImage is **withdrawn**; FUSE is only for the optional OpenRGB engine file.

## Alternatives considered

- `/usr/lib/chromaflow/` — easy to collide with multiarch libdir
- Running the repo script via pkexec during development — rejected for apply; dry-run stays unprivileged

### Critique

| Issue | Resolution |
|-------|------------|
| Developers cannot test apply without a package | Apply is stubbed; dry-run is the gate |
## Consequences

CI never installs polkit policy. `[HUMAN]` Mint VM for live apply later.
