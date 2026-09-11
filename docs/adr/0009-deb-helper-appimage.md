# ADR-0009: .deb helper vs AppImage

- **Status:** Accepted
- **Date:** 2026-09-11
- **Deciders:** ChromaFlow sprint 0

## Context

Polkit must execute a **pinned, root-owned** path. An AppImage extracted under `$HOME` is user-writable and must not be that path.

## Decision

- Helper install path: `/usr/libexec/chromaflow/install-support.sh`
- `.deb` (later) installs helper + polkit policy + optional daemon
- AppImage (later) ships GUI/CLI only and tells the user when the helper is missing
- This milestone ships repo copies only

## Alternatives considered

- `/usr/lib/chromaflow/` — easy to collide with multiarch libdir
- Running the repo script via pkexec during development — rejected for apply; dry-run stays unprivileged

### Critique

| Issue | Resolution |
|-------|------------|
| Developers cannot test apply without a package | Apply is stubbed; dry-run is the gate |

## Consequences

CI never installs polkit policy. `[HUMAN]` Mint VM for live apply later.
