# ADR-0022: Windows LPC IO via elevated helper (no WinRing0)

- **Status:** Accepted
- **Date:** 2026-10-21
- **Deciders:** ChromaFlow Windows port
- **Related:** ADR-0018 (PWM failsafe), ADR-0007 (MIT), ADR-0020 (native ports)

## Context

Motherboard fan PWM on Windows requires LPC / SuperIO register access. Shipping unsigned WinRing0-class drivers is a security and SmartScreen failure mode. Linking LibreHardwareMonitor (MPL) or Fan Control (proprietary) conflicts with MIT independence.

## Decision

1. Keep `chromaflow-gui` unelevated. LPC port IO lives only in `chromaflowd` / `chromaflow-lpc` (elevated service or helper).
2. Prefer **PawnIO** (or successor used by open tools) as the IO provider when available; document install in Support.
3. Core speaks a small named-pipe / fixture protocol (`CHROMAFLOW_LPC_MOCK`, `CHROMAFLOW_LPC_PIPE`). Mock fixtures keep CI green without hardware.
4. SuperIO register maps are original Rust from public chip documentation (ITE / Nuvoton first). Do not paste LHM or Fan Control source.
5. On service stop or crash, restore firmware/automatic fan mode (same contract as `pwm*_enable=2` on Linux).

## Consequences

- Full PWM on Windows needs the helper + IO provider; until then inventory shows an honest `windows_lpc` gap.
- NSIS packages GUI, CLI, OpenRGB.exe sibling, and service unit stubs under `Program Files\ChromaFlow`.
- **Shipped (ITE dual-chip MVP):** elevated `chromaflow-lpc` speaks `\\.\pipe\chromaflow-lpc`, loads PawnIO `LpcIO.bin`, and mirrors sensors into `%LOCALAPPDATA%\chromaflow\run\lpc-live\hwmonN` so `pwm_apply` / Auto-calibrate stay file-shaped. First target: Gigabyte dual-ITE (IT8689 + IT87952). Nuvoton maps remain a gap.
