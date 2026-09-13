# ADR-0013: Bundled OpenRGB engine as a sibling process

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow lighting engine

## Context

Keychron, GPU, and Aorus lighting on this product talk to the OpenRGB SDK on `127.0.0.1:6742`. Copying OpenRGB C++ into the MIT tree is forbidden ([ADR-0007](0007-mit-until-gpl-link.md)). Uninstalling the OpenRGB **desktop application** must not remove lighting if ChromaFlow still has a hidden engine file.

## Decision

- OpenRGB ships as a **separate GPL AppImage** (not in git, no Git LFS). ChromaFlow never pastes C++ and never links in-process.
- The Mint `.deb` installs that file at `/usr/libexec/chromaflow/OpenRGB.AppImage` (hashed fetch at package build). Pin HTTPS URL + sha256 + max size in `data/openrgb-engine.yaml`.
- `chromaflow daemon --sdk` (user unit `chromaflow-sdk.service`) reuses a live `127.0.0.1:6742` server or spawns the first allowed binary: `CHROMAFLOW_OPENRGB`, `/usr/libexec/chromaflow/OpenRGB.AppImage`, `~/.local/share/chromaflow/OpenRGB.AppImage`, then PATH `openrgb` only if [openrgb_sandbox.rs](../../crates/chromaflow-core/src/openrgb_sandbox.rs) reports no bwrap. Never Flatpak, bwrap, or Wine.
- Args: `--server --server-host 127.0.0.1 --noautoconnect --startminimized`. Do **not** set `QT_QPA_PLATFORM=offscreen`. Do **not** install an OpenRGB `.desktop`.
- Unprivileged. Pid file + SDK mutex + in-flight flag. Do not kill the sibling on GUI exit. `CHROMAFLOW_NO_SPAWN=1` in cargo/pytest.
- User-space install writes `.part`, enforces 60s / size / sha256, atomic rename, `chmod 0755`. No pkexec. User unit `chromaflow-sdk.service` is `chromaflow daemon --sdk`.

This supersedes “do not spawn” in [ADR-0008](0008-openrgb-localhost-probe.md). Localhost-only and no Wine stay.

## Alternatives considered

- In-process OpenRGB / relicensing to GPL — rejected (ADR-0007)
- PATH-only `openrgb` — rejected; uninstalling the distro GUI would break us
- `QT_QPA_PLATFORM=offscreen` — rejected; hides USB

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty binary | `ensure_sdk` no-op; Lighting shows `lighting.engineMissing`; research HID still lists |
| Download timeout / hash | 60s, size cap, `.part`, sha256, delete on fail; never exec a partial file |
| Inventory poll race | SDK mutex + `AtomicBool` + pid file; reuse live 6742 |
| FUSE missing | Spawn/install error includes `sudo apt install libfuse2`; GUI does not apt |
| GPL / supply chain | NOTICE + this ADR; pinned host + hash; no C++ in `crates/` |

## Consequences

[ADR-0008](0008-openrgb-localhost-probe.md) localhost probe still applies. [ADR-0009](0009-deb-helper-appimage.md) GUI AppImage is withdrawn. Mint PPA is a follow-up after local `dpkg -i`.
