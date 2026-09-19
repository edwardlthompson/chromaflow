# Changelog

All notable changes to ChromaFlow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Bootstrapped from [agent-project-bootstrap](https://github.com/edwardlthompson/agent-project-bootstrap) 1.4.0.

## [Unreleased]

## [1.9.0](https://github.com/edwardlthompson/chromaflow/compare/v1.8.0...v1.9.0) (2026-09-19)


### Added

* land Unreleased rail icons, Fusion HID apply, and product SBOM tags ([e03b3da](https://github.com/edwardlthompson/chromaflow/commit/e03b3daddb4c3275ad704cf13d66c223d1614054))
* prepare v0.1.0 native Mint cooling and lighting GUI ([aeec6d6](https://github.com/edwardlthompson/chromaflow/commit/aeec6d67f625caffb04a57d3ef8d27ccab4e304a))
* scaffold ChromaFlow sprint 0-1 inventory and support ([2289a9e](https://github.com/edwardlthompson/chromaflow/commit/2289a9ec9c0f03af6dc7d98b9b22c848a8ff79d8))


### Fixed

* **ci:** disable npm cache on the deb release job ([278a031](https://github.com/edwardlthompson/chromaflow/commit/278a031e88f597e4961e5dab66b23a572321c4c8))
* **ci:** install desktop npm deps before the release deb ([f405c3f](https://github.com/edwardlthompson/chromaflow/commit/f405c3f5749432bea2de52c987aaf863476da95e))
* **ci:** keep the deb release job zizmor-clean ([8192a1c](https://github.com/edwardlthompson/chromaflow/commit/8192a1c33c4834914b988436b7f12d19eeb94db0))
* do not use hashFiles in job-level CI ifs ([537878d](https://github.com/edwardlthompson/chromaflow/commit/537878dcb72f792df7ea3e9f12c561469ef4de47))
* load lighting helpers in CI without svelte/store ([c32765c](https://github.com/edwardlthompson/chromaflow/commit/c32765c9d351b25f66cf1cf385dbad2d340de479))
* satisfy clippy -D warnings in chromaflow-core ([a050412](https://github.com/edwardlthompson/chromaflow/commit/a05041215f5ede27ea2b10bc0c1cd3c2cd590efe))
* skip About-lego strip on pruned CLI stacks ([2a6b118](https://github.com/edwardlthompson/chromaflow/commit/2a6b1186b74c15f9b9695357d1c3bb3f54f0b40f))
* skip CodeQL Android analysis after examples/android prune ([fa60380](https://github.com/edwardlthompson/chromaflow/commit/fa60380de7e421854d04753ba33dd01184e3f266))
* skip pruned-stack CI jobs on the product repo ([5fca2f7](https://github.com/edwardlthompson/chromaflow/commit/5fca2f78999c382b864fa9818f098aef8bdba227))

## [0.2.2] - 2026-09-18

### Added

- Support **Check for updates**, plus a quiet check on each launch, against GitHub Releases
- Install `chromaflow_<version>_amd64.deb` only after the sha256 matches, through the pinned polkit helper
- The release workflow uploads that `.deb` when a GitHub Release is published

### Changed

- Template pin is 1.8.0. Product tags stay on the CHANGELOG version
- Desktop Svelte 5.57.1. Golden Path web patch/minor bumps only. CodeQL stays on `v4`

## [0.2.1] - 2026-09-14

### Changed

- Rail tabs keep all four pages mounted (`.tab-hidden`); OpenRGB inventory is not re-scanned on every click
- Color chips, wheel, sliders, and the effect menu apply on pick; Lighting has no Apply button
- Lighting whites and recents sit under the suggested chips; effect and speed sit under those
- Suggested chips match the hex preview square; channel +/- glyphs are centered
- Host Cycle All is not started from leftover session keys on Cooling; Solid Color stays put across tabs
- Support About includes Donate via Venmo
- All devices Hardware gauges card stays square; suggested color chips are squares
- Auto-calibrate shows a determinate progress bar per header
- Support About details shows the bundled product version and MIT license pointer
- PWM, Support uninstall, and device-report confirms use an in-app alertdialog; Cancel still does nothing
- First Cooling visit with writable PWM offers Quiet take-over before the board is a wall of checkboxes
- Rail, swatch, toast, picker, pill, extras, and gauge marker ease in 120–180ms unless reduced motion is on
- Lighting Apply paints the swatch immediately and toasts success; only that row’s Apply is busy
- Lighting and Cooling primary actions share the verb Apply, with distinct aria-labels
- Support first screen is two sentences plus Install all; extras and long errors sit in Details
- Type, space, and 44px hit tokens (`--fc-text-*` / `--fc-space-*` / `--fc-hit`); Hide and Clone use `.btn-secondary`
- Unused Lighting essays (`localhost`, Fusion/module prompts) are deleted; research help sits in closed Details
- Left rail uses large icons instead of tab names; the four buttons split the vertical space evenly and scale with window height
- Lighting lists Motherboard Fusion and CPU AIO when USB `048d:5702` is present, even if `liquidctl list` and hidraw are empty
- Fusion Apply paints analog + AIO over the HID serve when hidraw and `liquidctl list` are empty; CPU usage on those rows is not overwritten by Cycle All
- GitHub Release SBOM/tag gate matches CHANGELOG product version, not `.template-version`
- Cargo workspace, Tauri, desktop package, and `.deb` names follow CHANGELOG `0.2.0`
- Weekly health and open-PR BUILD_PLAN sync open a PR when they cannot push to protected `main`
- Header uses This machine / Watchdog pills instead of a status sentence; rail icons have labels; Lighting Apply no longer shows `broadcast FFFFFF`; Profiles is a form, not JSON
- Cooling and Support first-run copy is short; experimental GPU I2C is off until Advanced is checked; competing-controller uninstall sits in a closed details block
- All devices color picker grows sliders and gauges across the unused Lighting card width
- Lighting rows lead with the lamp name; protocol and hex stay behind Details
- Profiles Apply takes over fans with the named curve and paints lamps; Save still only writes `~/.config/chromaflow/`
- Desktop chrome uses `--fc-*` navy/yellow tokens; Golden Path teal is not imported into the product GUI

## [0.2.0] - 2026-09-14

### Changed

- Desktop Vite `8.2.2` → `8.3.0` (patch/minor `/update-deps`; CodeQL stays `@v4`)

### Fixed

- Lighting usage sliders share Cooling’s 400 ms `hardware_gauges` tick, including while Cycle All is on
- Cycle All GPU double-buffers RGB1, then re-enters static and save, so the last color holds until the flip; the first snap still arms with the full idle Apply sequence
- Cycle All and Breathing host-tick every lamp; Fusion HID commands time out in 400 ms so a stuck header cannot freeze the GUI, gauges, or the next hue
- Apply All SOLID retries until VIA reports effect 1 so the keyboard does not stay white
- Cooling temp graphs map 25–90 °C so a GPU in the high 20s is not flattened to the floor
- Quiet/Balanced/Performance/100% curves start at 25 °C so the graph and interp cover GPU idle
- All Devices only lists fills every lamp can run (Solid, Breathing, Cycle All, Combined); per-device menus group Solid, Rainbow, Rain and splash, and Hardware
- Apply to all keeps going when one lamp fails (GPU I2C), so motherboard and CPU AIO still get the color
- Combined on All devices paints every lamp; Cycle All is one host hue on Keychron, Arena, Prime, Fusion, and the GPU, started together so they stay in sync
- Apply to all paints Fusion once and the other lamps in parallel so devices are not left waiting on each other
- CPU AIO apply uses header analog (`led2`/`led5`/`led6`/`led7`/`led8`) plus a D_LED HID fill, and no longer overwrites motherboard analog
- GPU shroud apply writes ITE static RGB on NVIDIA I2C `0x68` when the adapter enumerates it
- Keychron host rainbow/spatial effects paint 9 VIA LEDs per report on the Q6 layout and keep Direct without re-entering firmware `0x17` every frame; Arena host effects drive all 4 HID zones
- Keychron Combined/usage fills use rgb_matrix SOLID (effect 1) after VIA GET showed custom `0x17` breathing on this Q6 HE; firmware speed is 0
- Motherboard CPU/usage applies immediately over liquidctl `sync` (host ticks no longer wait on eight serial CLI calls, and a failed apply is retried)
- Lighting SDK waits for the graphical session and `DISPLAY` so OpenRGB can see the GPU and Keychron after reboot (ADR-0019)
- OpenRGB sibling is reaped and respawned instead of left as a zombie; `nvidia-settings` is not called when GPU fan percent is unchanged
- Applying a solid Lighting color stops host gauge/usage effects on that device so they cannot overwrite the picker
- Keychron solid color now uses VIA rgb_matrix solid so the keys light, not only the GET_COLOR store
- Keychron Lighting apply paints over hidraw Direct (custom + per-key Solid + SET 0x0A) without OpenRGB
- Blank Lighting/Cooling window: WebKit custom protocol never ran ES modules, so the UI stayed `#121418`; production JS is now a classic IIFE, and WebKit also disables compositing (not only DMA-BUF) on this NVIDIA host
- 100% identify curve is held: the watchdog skips 5% step/hysteresis, and this session’s `chromaflowd` runs the tree binary so packaged Quiet cannot wind 100% back down
- Cooling fan cards refresh about 400 ms; a focused curve dropdown no longer pauses RPM; NVIDIA RPM is stale-while-revalidate
- Keychron Apply paints all 108 keys (per-key VIA type, one LED per SET) and shows the Q6 layout without OpenRGB; Fusion Apply sets analog led1–led8

### Added

- MSI RTX 4090 GPU RGB is listed from PCI; color apply writes I2C `0x68` when the ITE is present (OpenRGB SMBus was EIO)
- Lighting lists CPU AIO as its own Fusion row, separate from Motherboard Fusion analog onboard zones
- OpenRGB 1.0 detector identity catalog (`data/openrgb-device-index.csv`) as the G-LED-NATIVE backlog, with native Keychron VIA Direct writes and OpenRGB still the oracle/fallback (ADR-0020)

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
- Lighting helper tests import `ui.js` without `svelte/store` so CI Node can load host-effect modules
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
