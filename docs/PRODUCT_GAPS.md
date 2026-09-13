# Product gaps vs original brief

Catalog for Sprint 2+. Live board: [`BUILD_PLAN.md`](../BUILD_PLAN.md). Feature specs: [`docs/features/native-desktop.md`](features/native-desktop.md).

**Original brief** (2026-09-11): a **single Linux Mint desktop app** for fan/pump curves and RGB/LED, UX inspired by [Fan Control](https://github.com/Rem0o/FanControl) and [OpenRGB](https://github.com/CalcProgrammer1/OpenRGB), independent (NOTICE), no bundled `.ko`, GUI never root.

**Why it opened in a browser:** Sprint 1 shipped a Tauri 2 host (`apps/desktop/src-tauri`) plus a Vite UI. This Mint host has WebKit/GTK **runtime** but not the **`-dev`** packages, so `chromaflow-gui` cannot compile here. First-run used `vite preview` in Cursor/Chrome. That is a **dev fallback**, not the product. Fan Control and OpenRGB are native windows (no URL bar). ChromaFlow’s native shell is still Tauri (ADR-0006): a **.desktop window**, not a browser tab.

## Gap list

| ID | Original brief | Current | Sprint |
|----|----------------|---------|--------|
| G-APP | Standalone Cinnamon app (`chromaflow` / `chromaflow-gui`) | Vite tab; no `.desktop`; Tauri crate exists, local compile blocked on GTK/WebKit **dev** | 2 |
| G-UX | Look/act like Fan Control + OpenRGB (graphs, device tree, curve editor) | Cooling: Fans / Pumps / Temps / Curves; temp cards for CPU/GPU/RAM/Disk/Combined + mix; AIO checkbox on every control card | 20 |
| G-COOL | hwmon + liquidctl AIO + GPU fans; curves, hysteresis, min/max; daemon on boot | Schema v2 + NVIDIA GPU fans + ITE names + AIO unit; `it87-dkms` first; optional apt `liquidctl` (not installed on this host) | 20 |
| G-LED | OpenRGB device list, modes, colors, per-LED, sync, profiles | SDK protocol/LED count/color/modes per device + Kelvin picker + per-LED matrix; liquidctl Fusion fallback; Arena 7 HID; Prime Neo hidraw 0x62; Fusion USB 048d:5702 treated as the Aorus chip | 4, 7, 8 |
| G-DET | Detect hidraw/i2c/module/Windows-only gaps; Install + Rescan | `gaps[]` exists; Support dry-run selects YAML ∩ lspci/DMI/loaded/`i2c-dev` (not the full dump) | 5 |
| G-SUP | Polkit script: apt, udev, groups, allowlist `modprobe`, JSON, logout/reboot | One-click pkexec `--apply` via pinned helper; optional `--advanced` I2C | 5 |
| G-PROF | Profiles bind a fan-curve set + an RGB profile in `~/.config/chromaflow/` | Schema v1 JSON round-trip; names only; nothing applied to PWM | 5 |
| G-PACK | `.deb` + Start menu; OpenRGB engine file in the same package | `scripts/build-chromaflow-deb.sh` stages `chromaflow_*.deb` plus `/usr/libexec/chromaflow/OpenRGB.AppImage`; `chromaflow daemon --sdk` (ADR-0013) | 5 |
| G-SAFE | Never silent 0% PWM; watchdog + `ExecStopPost` failsafe | `chromaflow daemon --watchdog` + unit `ExecStopPost`; firmware enable (2); 0% needs confirm | 15 |
| G-BOOT | Optional `/etc/modules-load.d/chromaflow.conf` via polkit | Dry-run plan only (`apply: false`) | 5 |
| G-I18N | English UI, design tokens, Settings/About | Product UI hardcodes English + raw hex; Golden Path tokens live in `examples/web` | 3 |

This machine’s live `rescan` (Gigabyte X570S AORUS MASTER + RTX 4090 hybrid):

| Control | Probe | Notes |
|---------|-------|--------|
| FAN4 / FAN5_PUMP / FAN6_PUMP | `it87952` pwm1–3 | Secondary ITE only; pwm3 ~1660 RPM; pwm1/2 0 RPM at high duty (no tach) |
| GPU fan 0 / GPU fan 1 | `nvidia-settings` | Hybrid AIO + card fan; idle 0% / 0 RPM; take-over needs Coolbits |
| AIO unit | Cooling checkbox | Default members: FAN5_PUMP + FAN6_PUMP + both GPU fans |
| CPU_FAN / SYS_FAN1–3 / CPU_OPT | `it8689_*` pwm1–5 after host `it87-dkms` | Two tachs live (~2000 / ~1550 RPM); likely the case fans. CPU AIO stays on its own controller |
| FAN4 / FAN5_PUMP / FAN6_PUMP | `it87952_*` pwm1–3 | Secondary ITE; extra MMIO tachs may appear |
| USB AIO | none | No NZXT/Corsair/Asetek HID; `liquidctl` not installed |

**Deferred:** Fan Control File `.sensor` mixes (plugin-style). Mix + is Max/Min/Average/Sum/Subtract/Offset/Time average of hwmon + Lighting gauge ids only.

## Locked (not gaps)

- No Fan Control or OpenRGB source copies (proprietary Fan Control; GPL stop-and-ask).
- No GUI as root; no bundled `.ko`; allowlist-only `modprobe` names.
- MIT until an in-process OpenRGB link (ADR-0007).
- Tauri + Rust, not Avalonia (ADR-0006). Close UX gaps **inside** the native window.

## Parallelization

1. **Sequential lock:** native launch contract (`docs/features/native-desktop.md`) before window chrome.
2. **Sprint 2 parallel:** `apps/desktop/src-tauri/` vs `packaging/` (non-overlapping).
3. **`agent_count_target`:** 2 for Sprint 2; later sprints 2–3 after each sequential lock.
4. **Dry-run:** `python3 scripts/agent-run.py check-build-plan-parallel`

### Critique

| Issue | Resolution |
|-------|------------|
| Browser mistaken for the app | Sprint 2: Cinnamon `.desktop` + `cargo run -p chromaflow-desktop`; docs say Vite preview is not product smoke |
| Null inventory / empty hwmon | Fixture fallback; Tauri `inventory` invoke; Cooling lists all chips, flags invalid temps |
| OpenRGB timeout | 6742 localhost only, short timeout, Lighting copy “No Linux backend yet”; no fake devices |
| Race: two fan daemons | `conflicts[]` banner; no PWM until Sprint 6 + explicit confirm |
| Unhandled Tauri compile fail | `[HUMAN]` apt of CI WebKit/GTK **dev** packages; CI already installs them |
| PWM 0% | ADR-0018; `percent_to_duty` Err unless `allow_zero`; Cooling `cooling.zeroAsk` |
