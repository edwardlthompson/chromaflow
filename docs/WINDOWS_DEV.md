# Windows development

ChromaFlow on Windows is Tauri 2 + Rust + Svelte (same stack as Linux Mint).

## Prerequisites

- Rust stable (MSVC toolchain)
- Node 22
- WebView2 Runtime
- Visual Studio Build Tools (C++ workload) for Tauri

## Quick start

```powershell
cargo test --workspace --exclude chromaflow-desktop
cd apps\desktop
npm ci
cargo run -p chromaflow-desktop --no-default-features
```

CLI inventory:

```powershell
cargo run -p chromaflow-cli -- sensors
cargo run -p chromaflow-cli -- support
```

## LPC / SuperIO bridge (`chromaflow-lpc`)

Build and install the elevated helper (PawnIO must already be running):

```powershell
cargo build -p chromaflow-lpc --release
cargo build -p chromaflow-cli --release
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows-runtime.ps1
```

That copies `chromaflow-lpc.exe` + `LpcIO.bin` to `Program Files\ChromaFlow`, registers scheduled task **ChromaFlowLPC** (SYSTEM, `--serve` at startup), and starts it. Unelevated core pings `\\.\pipe\chromaflow-lpc` (override `CHROMAFLOW_LPC_PIPE`). Shadow hwmon lives under `%LOCALAPPDATA%\chromaflow\run\lpc-live\`.

Reboot only if the **PawnIO** service will not start; the LPC task alone usually does not need a reboot.

Smoke:

```powershell
cargo run -p chromaflow-cli -- sensors
# Support item superio_lpc should be Ready when pipe ping succeeds.
```

## LPC / PWM mock

Without the elevated helper + PawnIO (ADR-0022), use an explicit lab fixture (never auto-enabled by leftover files):

```powershell
$env:CHROMAFLOW_LPC_MOCK = "1"
cargo run -p chromaflow-cli -- cooling --takeover
```

Support → Apply clears the lab marker. Lab PWM is **file-only** — Auto-calibrate will not spin chassis fans until `chromaflow-lpc` answers on the named pipe.

## NSIS installer

```powershell
.\scripts\build-chromaflow-nsis.ps1
```

Output lands under `apps/desktop/src-tauri/target/release/bundle/nsis/` (or workspace `target/release/bundle/nsis/`).

## Paths

| Role | Location |
|------|----------|
| Config / profiles | `%APPDATA%\chromaflow` |
| Data / private lighting engine | `%LOCALAPPDATA%\chromaflow\engine\OpenRGB\` |
| Runtime / owned PWM | `%LOCALAPPDATA%\chromaflow\run` |

Override with `CHROMAFLOW_CONFIG_HOME`, `CHROMAFLOW_DATA_HOME`, `CHROMAFLOW_OWNED`.

## Lighting engine

Support → Apply seeds a **private** OpenRGB tree (not a standalone app). Override source with:

```powershell
$env:CHROMAFLOW_OPENRGB = "C:\path\to\portable\OpenRGB.exe"
cargo run -p chromaflow-cli -- support --apply
```

The GUI starts the engine hidden on `127.0.0.1:6742` (`CREATE_NO_WINDOW`). Never use Windows `timeout.exe` as a process wrapper — it steals focus.

**Uninstall standalone OpenRGB** from Settings → Apps if you installed it earlier. ChromaFlow’s private copy under `%LOCALAPPDATA%\chromaflow\engine\OpenRGB\` (or `Program Files\ChromaFlow\engine\`) is the only lighting engine you need. Do not uninstall ChromaFlow itself.

Support/spawn also writes OpenRGB settings so the private engine never prompts:

- `LogManager.dialog_no_show_hashes` — WinRing0/PawnIO I2C warning
- `UserInterface.run_zone_checks: false` — no “Resize the zones” dialog
- SDK `RESIZE` to **32 LEDs** (or device max) for any linear zone still at size 0

Motherboard SMBus RGB still needs PawnIO + an elevated one-time setup if you want DRAM/onboard I2C lamps; USB/GPU SDK devices work without it.

Release GUI embeds Vite after `npm run build` in `apps/desktop` (crate default feature `custom-protocol`). Plain `cargo build --release -p chromaflow-desktop` is enough; without that feature the WebView loads `http://127.0.0.1:1420` and Edge shows a refresh/error page. Dev against live Vite needs `--no-default-features`.

## Safety

- Do not run `chromaflow-gui` elevated.
- PWM 0% still requires explicit confirm; failsafe restores firmware control when the watchdog stops.
- Release `chromaflow-gui.exe` is `windows_subsystem = "windows"` (no console).

## Autostart

- Optional registry snippet: `packaging/windows/autostart.reg`
- Watchdog service: `scripts/install-chromaflowd-windows.ps1` (admin)

## Runtime install (PawnIO + chromaflowd + chromaflow-lpc)

Elevated (UAC):

```powershell
cargo build -p chromaflow-cli --release
cargo build -p chromaflow-lpc --release
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows-runtime.ps1
```

That installs/upgrades **PawnIO** (`winget` id `namazso.PawnIO`), copies `chromaflowd.exe` + `chromaflow-lpc.exe` + `LpcIO.bin` to `Program Files\ChromaFlow`, registers the watchdog service and **ChromaFlowLPC** task, and clears the LPC **lab** marker.

Support marks `superio_lpc` Ready only when the named pipe answers ping. If the binary is present but the pipe is down, Cooling shows `lpc_pipe_down` — not fake fans.

## Acceptance

See [`docs/features/windows-desktop.md`](features/windows-desktop.md).
