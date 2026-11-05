# ADR-0023: GPL-3.0-or-later + private OpenRGB lighting engine

- **Status:** Accepted
- **Date:** 2026-10-21
- **Deciders:** Product owner (HUMAN) + Windows lighting milestone
- **Supersedes:** [ADR-0007](0007-mit-until-gpl-link.md)

## Context

ADR-0007 kept ChromaFlow MIT and treated OpenRGB as an optional separate program. Users on Windows still had to install or point at a standalone OpenRGB, and Windows `timeout.exe` / console child processes stole focus. The product goal is lighting that feels built-in, with a license that permits tighter OpenRGB integration later (including in-process).

OpenRGB is GPL-2.0-or-later; liquidctl is GPL-3.0-or-later. Shipping a private bundled engine and unlocking future linkage requires copyleft on the ChromaFlow product itself.

## Decision

1. Relicense **ChromaFlow** to **GPL-3.0-or-later** (root `LICENSE`, Cargo workspace `license`, Winget/NSIS metadata, UI About string).
2. Ship OpenRGB as a **private lighting engine** under `…\ChromaFlow\engine\OpenRGB\` (no Start Menu entry, no user-facing “OpenRGB app”). Lifecycle is owned by ChromaFlow (`--server --server-host 127.0.0.1 --server-port 6742 --noautoconnect` + `CREATE_NO_WINDOW`). Never pass `--startminimized` (that implies `--gui` and a tray icon).
3. Keep talking to the engine over localhost SDK `127.0.0.1:6742` this milestone. In-process C++ link remains a follow-on under this license.
4. All Windows child processes must use `win_cmd` (`CREATE_NO_WINDOW`); never invoke Windows `timeout.exe` as a command wrapper.

## Alternatives considered

- Stay MIT + separate OpenRGB — rejected (standalone UX, license wall for in-process)
- Immediate in-process OpenRGB C++ link — deferred (Qt/build cost); license change unlocks it
- Bundle without relicense — rejected (NOTICE already warned linking needs discussion)

### Critique

| Issue | Resolution |
|-------|------------|
| MIT sacred drift | Explicit HUMAN direction; LICENSE + NOTICE + this ADR in one change |
| Console focus steal | `nvidia_smi` Windows path + `win_cmd` on tasklist/sc/spawn |
| Users still see OpenRGB UI | No shortcuts; hidden server flags; Support copy says “lighting engine” |
| GPL vs App Store / proprietary plugins | Product stays FOSS; no proprietary SDK on production path |

## Consequences

- Downstream redistributors must comply with GPL-3.0-or-later.
- Native HID ports remain first-class; the private engine covers motherboard/GPU controllers OpenRGB already supports.
- Future in-process OpenRGB or liquidctl linkage no longer blocked by MIT.
