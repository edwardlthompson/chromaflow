# Changelog

All notable changes to ChromaFlow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Bootstrapped from [agent-project-bootstrap](https://github.com/edwardlthompson/agent-project-bootstrap) 1.4.0.

## [Unreleased]

## [0.1.0] - 2026-09-13

### Added

- Locked ChromaFlow mark (hue ring + gold fan with motion trails) and glass-floor brand shot in the README; color Linux panel icon plus a Windows `.ico` for a future taskbar/Start build
- Login autostart (`/etc/xdg/autostart` + user `chromaflow-gui.service`), Cinnamon favorite launcher, and a color tray icon (close hides to tray)
- Quiet / Balanced / Performance **Apply to all** for every on-board fan and pump; cooling names, curves, hide/show, and take-over persist in `curves.json`
- Product identity (ChromaFlow), NOTICE, architecture/hardware/support docs, ADRs 0006–0010
- Read-only `chromaflow` inventory CLI and Support `--dry-run` script (no PWM writes, no live modprobe)
- Tauri 2 + Svelte stub window (Cooling, Lighting, Profiles, Support); `support_dry_run` invoke; CI WebKit compile on Ubuntu 22.04/24.04
- Desktop UI loads this-machine inventory (Tauri `inventory` or `live-inventory.json`) and shows °C plus every hwmon chip
- Product gap catalog vs original Fan Control/OpenRGB brief (`docs/PRODUCT_GAPS.md`); BUILD_PLAN Sprint 2+ is a native Cinnamon window, not a browser tab
- Native `chromaflow-gui` window on Cinnamon (Tauri decorations + `.desktop`); GTK/WebKit **dev** packages installed on this host
- Cooling page groups chips, shows °C / hysteresis fields, and samples a live graph (still no PWM writes)
- OpenRGB SDK client lists controllers on `127.0.0.1:6742` (timeout, no fake devices)
- Machine-specific Support dry-run (YAML ∩ lspci/DMI/loaded/`i2c-dev`) plus a modules-load.d **plan** that is never written
- Profiles schema v1 in `~/.config/chromaflow/` (curve set + RGB name; no PWM apply)
- `chromaflow daemon --dry-run` inventory watch (no PWM; `CHROMAFLOW_DAEMON_ONCE=1` for tests)
- Desktop UI uses a Fan Control-like cooling layout (Controls + Curves navy cards, yellow graph points; still no PWM writes)
- Local `chromaflow-helper` .deb pins Support apply at `/usr/libexec/chromaflow/install-support.sh`
- Support **Uninstall competing controllers** stops and `apt purge`s fancontrol, CoolerControl, fan2go, thinkfan, NBFC, OpenRazer, and ckb-next via pinned `manage-competitors.sh` (not OpenRGB or ChromaFlow)
- VID-specific hidraw udev (Fusion, Keychron Q6 HE, SteelSeries Prime, Arena 7) with `MODE=0660` `GROUP=plugdev`; gap `openrgb_sandboxed` when OpenRGB runs under bwrap
- Localhost OpenRGB SDK color apply (`SET_CLIENT_NAME` / `UPDATE_LEDS`) plus liquidctl Fusion fallback (ADR-0011); no PWM or hidraw writes
- Lighting hex/swatch apply in the native window (Tauri IPC `remote` URLs + command allowlist); Cooling min/max move the display curve
- Lighting hue ring + inner SV square (OpenRGB-style), independent R/G/B, H/S/V, and Kelvin (CCT) sliders with numeric fields and steppers, suggested chips, and the last two manually picked colors
- Extra-kernel **Install** per item and **Install all** via pkexec `--only`; failed extras show an error instead of a silent skip
- Support Extra kernel support is the install checklist for fans and RGB I2C: ITE/Nuvoton Super I/O, `k10temp`, AMD SMBus, DIMM SPD (`jc42`/`spd5118`), plus experimental RGB I2C
- Lighting lists protocol, LED count, and current color per device; click a swatch to open that device’s color wheel; All devices has its own wheel and apply
- OpenRGB SDK firmware effects (`UPDATE_MODE`) listed per controller; per-LED keyboard/matrix layout from the SDK map (`UPDATE_SINGLE_LED`)
- Mint `chromaflow_0.1.0_amd64.deb` ships GUI, CLI, Cinnamon menu, helper, and the hashed OpenRGB AppImage at `/usr/libexec/chromaflow/`. `chromaflow daemon --sdk` keeps the sibling on `127.0.0.1:6742`. Research HID list and GitHub device form.
- One `chromaflow-gui` per user session: a second launch focuses the existing window (`chromaflow-gui.sock`)
- Lighting LED boards sit on the picker row as collapsible 4:1 cards (keyboard aspect, packed motherboard/other LEDs); expanded boards poll OpenRGB LED colors as fast as the SDK round-trip allows
- Keychron Q6 HE LED layout polls every key over VIA HID (`0xA8`/`0x09`) and paints with `requestAnimationFrame` at the monitor refresh rate (ADR-0015)
- Lighting picker uses three equal squares (wheel, sliders, hex/swatches/effect) that wrap; GUI restores the last window size from `~/.config/chromaflow/window.json`
- Slider and hex tooltips (RGB, HSV, Kelvin) plus full-width square swatches in the hex cube
- Host Direct hardware gauges (`CPU + GPU temp`, `RAM used`, `Disk used`) paint every LED green→yellow→red from hwmon/`meminfo`/`df` (ADR-0017; no OpenRGB plugin)
- Lighting shows a picker-row hardware cube (same size as the color wheel) with Temp/Usage toggle; Disk °C from nvme hwmon; RAM °C from jc42/spd5118 when present
- Cached `nvidia-smi` GPU °C (2 s) when hwmon has no GPU group (ADR-0017)
- PWM watchdog (`chromaflow daemon --watchdog`) plus firmware failsafe (`pwm*_enable=2`, ADR-0018)
- Lighting **Report** asks for confirm, then dims that device after the GitHub form opens
- Cooling Fans / Pumps / Temps / Curves board (schema 2 recipes, mix sensors, Quiet/Balanced/Performance graphs, auto-calibrate duty%↔RPM; never silent 0%)
- Cooling detects NVIDIA GPU fans (`nvidia-settings` fan:0/1), names Gigabyte IT87952 headers FAN4/FAN5_PUMP/FAN6_PUMP and IT8689 headers CPU_FAN/SYS_FAN*, groups AIO members as one take-over unit, and reports `ite_primary_missing` only while the primary Super I/O is absent
- Support Extra kernel support lists frankcrawford `it87-dkms` first (present = live `it87` srcversion matches DKMS, not in-tree); Install uses apt if the package is already in sources and never adds CoolerControl
- Optional apt `liquidctl` as Extra kernel support (USB AIO). Install skips extras that are already present (no re-modprobe / DKMS reload). Cooling temp cards show usage % plus 60 s usage (green→red) and temp (blue→red, 30–90 °C) graphs with a 0–100% / 30–90 °C wireframe (vertical bar every 15 s). Temps **+** adds a mix; Curves **+** adds a named custom curve. `chromaflow cooling --takeover` applies Balanced (min 20%) and auto-calibrates tach headers. The `.deb` ships hidraw/i2c udev; Install all is one pkexec (`auth_admin_keep`).

### Changed

- Lighting Uncontrolled devices is a short empty line when there are no leftover HID lamps
- Cooling Temps are CPU/GPU/RAM/Disk/Combined/mix cards with usage % and 60 s graphs; mix checkboxes live behind Temps **+**; Quiet/Balanced/Performance stay locked and **Clone** (or Curves **+**) makes an editable copy with °C/% labels inside the SVG; Fans, Pumps, Temps, and Curves tiles are all `22rem` like a fan card; one Cooling **Auto-calibrate** sweeps every writable fan then pump; Cooling inventory skips OpenRGB/`ensure_sdk`; every fan/pump card has an AIO unit checkbox; temp dropdowns stay inside the card; top bar and left tab rail do not scroll; OpenRGB/HID lighting ticks run only on the Lighting tab
- Lighting top card is Devices; OpenRGB SDK reachable status and 127.0.0.1:6742 are not shown
- Lighting Extra kernel support lives on Support; Devices lists only controllable lamps and omits Fusion/radiator notes
- Lighting hardware cube Temp/Usage and B–R/G–R are switches (Temp with blue–red by default); LED color follows both and the palette is saved in the session
- Lighting hardware effects are named CPU, GPU, Combined, RAM, and Disk; LED color follows the cube Temp/Usage toggle
- Lighting Apply effect is a host Direct engine (QMK-style math) on every OpenRGB device so each LED color is known; firmware `UPDATE_MODE` is not used for that catalog (ADR-0016)
- Lighting effect picker is the dropdown itself (no “Effect” caption); a Speed slider (QMK 1–255) drives the running host animation
- Host motion effects push OpenRGB at 10 Hz; gauges and solid fills push only when the color changes (2 Hz max); sensor reads are 1 Hz temps-only with a 30 s disk cache
- Inventory poll is 15 s and skipped while Lighting is open so the 108-key board is not rebuilt on every OpenRGB DATA dump
- Native GUI restores the last tab and each device’s host effect/color from `~/.config/chromaflow/session.json` (Lighting unmounts off-tab so Cooling is not painting the 108-key board)
- Support Extra kernel support holds the experimental RGB I2C checkbox; Preview plan and This machine are removed (dry-run still runs on open)
- Lighting Uncontrolled devices has a Report button per leftover HID (GitHub `device` issues). Status says fan duty is not written until take-over
- Cooling take-over confirms, then the watchdog writes duty; failsafe restores firmware `pwm*_enable=2`
- Cooling take-over enables user `chromaflowd` and Support can install plugdev PWM ACL (`pwm_acl`) so duty survives GUI close
- Cooling and Support treat only **running or enabled** fan/RGB daemons as conflicts (leftover unit files and purged packages do not block PWM)

### Fixed

- Cooling hydrates `curves.json` after this-machine inventory arrives, so Quiet **Apply to all** is not overwritten by the sample fixture and survives quit/relaunch
- Cooling no longer always shows that take-over is blocked; that line is only when a live competitor is present. Fan/pump/temp/curve tiles are `22rem` so AIO and %/RPM stay on the card.
- Inventory lists IT87952 `pwm4`/`pwm5` even when duty reads ENODATA until `pwm*_enable=1`, pairs `fanN` with `pwmN`, and `chromaflowd` passes `DISPLAY=:0` to `nvidia-settings` so GPU fans stay under the watchdog after the GUI closes.
- Cooling hides fan/pump cards with no tach (Hidden row; NVIDIA GPU cards stay on the board even at idle 0 RPM).

- Installed `chromaflow-gui` loads the Svelte UI: Vite `base: "./"`, the `.deb` runs `npm run build` then `cargo --features custom-protocol` (otherwise WebKit opens `127.0.0.1:1420` and shows connection refused), and Linux sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` so NVIDIA DMA-BUF is not a black surface
- Installed Lighting no longer asks you to fetch the engine or “start a native server yourself”: the `.deb` includes `/usr/libexec/chromaflow/OpenRGB.AppImage` and starts `chromaflow daemon --sdk`
- Curve graphs keep °C/% labels inside a 7.25rem SVG (`preserveAspectRatio="xMidYMid meet"`) instead of HTML overlays clipped by the card
- Watchdog PWM apply follows `/sys/class/hwmon` device symlinks (class paths used to look like an escape)
- Support Install for NCT6775 I2C now loads in-tree `nct6775-i2c` (there is no `i2c-nct6775` module). If NVIDIA GPU I2C is already loaded, both RGB I2C extras show present. ITE Super I/O counts Nuvoton present. `linux-modules-extra` is present when the hwmon modules are already in the running kernel. Masked `coolercontrold` leftovers are not a Cooling conflict.
- Effect dropdowns restore the last applied host effect per device instead of snapping to Solid Color after a relaunch or tab change
- Host effects paint the layout locally and cache the OpenRGB/liquidctl probe for 15 s so inventory does not freeze keys and the LED board together
- Lighting skips identical host-frame paints and does not poll inventory while the Lighting tab is open, which stopped a remaining Cinnamon hitch
- Host USB/IPC no longer runs at display refresh: idle Lighting uses a 1 s timer, duplicate gauge frames are not written, and `lighting_sync` does not serialize LED arrays back to the UI while pushing
- Host effects keep Apply state across Lighting remounts, stamp written LED colors onto the layout (so the board is not HID-black), and chromatic animations ignore a live black/white picker so Rainbow Moving Chevron actually moves when Speed changes
- OpenRGB `UPDATE_LEDS` `data_size` now equals the full SDK payload so pipeline/git2478 accepts color writes (was `2+4*n`, rejected as invalid size 10/22/438)
- Aorus/Fusion color apply also runs `liquidctl` after the SDK write so motherboard LEDs are not a silent no-op
- SteelSeries Arena 7 color apply via vendor HID output report `0x06` (ADR-0012)
- Kelvin slider can reach 2000–6500 K (Tanner invert, not McCamy)
- Per-LED keyboard map uses live SDK colors and ANSI-ish widths for Space/Shift/Enter
- RGB Fusion 2.0 USB is not listed as uncontrolled when the Aorus/liquidctl path already drives it
- Keychron Q6 HE map uses ANSI 100% geometry (Enter/numpad columns line up); firmware effects handshake OpenRGB protocol 3 and set speed/brightness
- Apply effect sends `UPDATE_MODE` twice (re-read DATA in between) and drops the preview TCP session so the first click is not a white Direct flash
- LED layouts poll real Keychron per-key HSV over hidraw and, while the board is open, Direct-stream so that readback matches the keys (ADR-0015)
