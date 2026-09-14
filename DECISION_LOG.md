# Decision Log

> Chronological register of major technical trade-offs, accepted architectures, and rejected alternatives.
> **Treat past entries as immutable history; append only.**

## Format

```markdown
### YYYY-MM-DD — [Title]
- **Status:** Accepted | Rejected | Superseded
- **Context:** ...
- **Decision:** ...
- **Alternatives considered:** ...
- **Consequences:** ...

```

### 2026-09-14 — Product Release SBOM tag is CHANGELOG version
- **Status:** Accepted
- **Context:** Template `release.yml` required GitHub tags to equal `.template-version` (1.4.0), so product `v0.2.0` could not attach SBOM via the workflow.
- **Decision:** `scripts/product-release-version.sh` reads the first CHANGELOG `[X.Y.Z]`. Tag gate and SBOM default tag use that. `.template-version` stays the bootstrap pin. Do not merge a template 1.5.0 Release Please PR.
- **Alternatives considered:** Bump `.template-version` to 0.2.0 (rejected: that is the template pin). Use Cargo workspace 0.1.0 (rejected: GitHub Release is 0.2.0).
- **Consequences:** `workflow_dispatch` / tag `v0.2.0` can attach SBOM. PWM unchanged.

### 2026-09-14 — Fusion Apply without hidraw/liquidctl
- **Status:** Accepted
- **Context:** USB `048d:5702` is claimed by `fusion-hid.py serve` (usbfs), so hidraw is missing and `liquidctl list` is empty. Motherboard Apply still spawned liquidctl and failed. Cycle All’s lamp thread kept painting Fusion, so CPU usage never stuck.
- **Decision:** `set_device` writes Fusion HID first (`soft` / `digital` / `uniform`); liquidctl analog is optional. Cycle All is host `fixed`, not firmware `color-cycle`. The Cycle All lamp thread no longer writes Fusion; Lighting ticks Fusion even while other lamps cycle.
- **Alternatives considered:** Rebind usbhid (rejected: serve already owns usbfs). Firmware color-cycle for Fusion (rejected: unsynced, and it hid CPU usage).
- **Consequences:** Motherboard and CPU AIO follow CPU usage / Apply while USB is present. PWM unchanged.

### 2026-09-14 — Fusion rows follow hidraw 048d:5702
- **Status:** Accepted
- **Context:** OpenRGB is opt-in, `liquidctl list` is empty, and the ITE has no hidraw node, so Motherboard Fusion and CPU AIO vanished even though USB `048d:5702` is present.
- **Decision:** Inventory injects Fusion from `/sys/bus/usb/devices` when hidraw is missing. `fusionTargets` lists both rows from that USB id or a liquidctl Fusion name. Native apply is still liquidctl + fusion-hid.
- **Alternatives considered:** Keep liquidctl-list-only (rejected: empty list hid the lamps). Show the ITE HID as one Uncontrolled leftover (rejected: Apply path already exists).
- **Consequences:** Lighting shows analog + AIO whenever the Fusion USB device enumerates. PWM unchanged.

### 2026-09-14 — Automate leftover 0.2.0 HUMAN rows
- **Status:** Accepted
- **Context:** `/ship` left Actions PR permission, GitHub Release SBOM, and gtk-rs Medium on HUMAN.
- **Decision:** Enable Actions write + PR approval via API. Publish GitHub Release `v0.2.0` on `main` with CycloneDX + OpenVEX (template `release.yml` still keys off `.template-version`, so do not dispatch it for product tags). Keep glib 0.18.5 until Tauri gtk-rs 0.20.
- **Alternatives considered:** Merge a Release Please 1.5.0 PR (rejected: that is template version). Bump gtk-rs now (rejected: breaking Tauri 2).
- **Consequences:** https://github.com/edwardlthompson/chromaflow/releases/tag/v0.2.0 holds SBOM assets. Do not merge a template 1.5.0 RP PR. PWM unchanged.

### 2026-09-14 — Left rail uses even-height icons
- **Status:** Accepted
- **Context:** Tab names wasted rail width and did not grow with the window. `/ship` left GitHub Actions PR permission, Release SBOM, and gtk-rs/glib Medium on the table.
- **Decision:** Four SVG buttons live in `.rail-slot` cells. Header is 3rem; each tab is `calc((100vh - 3rem) / 4)` so WebKitGTK tracks window height without depending on nested percentage grid. Icons scale to 55% of each slot. Incomplete 0.2.0 items stay as Sprint 34 HUMAN rows.
- **Alternatives considered:** Keep text labels (rejected: user asked for large icons). `flex-grow` on `<button>` / nested flex / `1fr` on an absolutely positioned rail (rejected: WebKitGTK leaves them packed at the top). Cap icon `max-height` (rejected: they would not grow with a tall window).
- **Consequences:** Session tab ids stay Cooling/Lighting/Profiles/Support. PWM unchanged.

### 2026-09-14 — Ship 0.2.0 with Medium glib / unmaintained unic on local audit
- **Status:** Accepted
- **Context:** `/ship` `upd audit --check` exits 6 on GHSA-wrw7-89jp-8q8g (Medium, `glib` 0.18.5) and unmaintained `proc-macro-error` / `unic-*` with no `fixed_version`. There is no High or Critical. `npm audit --audit-level=high` is clean. `glib` 0.20.0 is gtk-rs 0.18→0.20, not a patch/minor for Tauri 2.
- **Decision:** Do not bump gtk-rs this release. Local `pre-release-gate.sh --local` stays red on `upd audit --check`. Feature-gate, license, and version passed. PWM unchanged.
- **Alternatives considered:** `--fix-audit` to glib 0.20 (rejected: breaking gtk-rs). `--no-fail` (rejected: disables the gate).
- **Consequences:** GitHub Dependabot/CodeQL remain the High+ inbox. Revisit gtk-rs when Tauri allows it.

### 2026-09-14 — GPU Cycle All double-buffers RGB1 then static+save
- **Status:** Accepted
- **Context:** RGB1 writes do not show until a mode latch, so they are already a back buffer. Idle-first Apply snaps walk but drop static (rainbow leak). Save without re-entering static stays stuck. Zone blocks while in static flash.
- **Decision:** First `set_snap` still arms with idle + RGB + blocks + static + save. Later snaps write RGB1, then static `0x13` + save `0x3F` in one `i2cset` batch (no idle, no zone blocks, no `i2ctransfer`). Fallback to the full sequence if the batch fails. `GPU_MS` stays 500.
- **Alternatives considered:** Idle every snap (rejected: that is the pop). RGB1 or save-only (rejected: stuck). Keep zone blocks on the flip (rejected: they flash).
- **Consequences:** Re-enable Cycle All. If the jewel freezes, this ITE still needs idle to latch and there is no hidden bank. PWM unchanged.

### 2026-09-14 — Restore GPU Cycle All saved static snaps
- **Status:** Superseded
- **Context:** Program-then-save without idle (`i2ctransfer` + `0x3F`) left the shroud stuck, same as RGB1-only. This ITE only shows a new host color after idle + static + save.
- **Decision:** `set_snap` always uses the full Apply sequence every 500 ms. Pops from idle are accepted until a real double-buffer exists. No `i2ctransfer`.
- **Alternatives considered:** Keep no-idle commit (rejected: stuck twice). Firmware rainbow (rejected: unsynced).
- **Consequences:** Shroud should walk again. Color pops during idle remain. PWM unchanged.

### 2026-09-14 — GPU Cycle All programs RGB then save, no idle
- **Status:** Superseded
- **Context:** Full Apply snaps walk the wheel but idle `0x1C` plus bytewise RGB leave a gap where leftover colors pop. RGB1 with no save does not stick. The jewel should keep the last static color until the next color is committed.
- **Decision:** After the first arm, `commit_next` writes RGB1 and BGR blocks in one `i2ctransfer` (no idle, no mode rewrite), then save `0x3F` with no extra delay. Apply and the first snap still use idle + static + save. Fallback to the full sequence if the transfer fails. `GPU_MS` stays 500.
- **Alternatives considered:** Idle every snap (rejected: that is the gap). RGB1 with no save (rejected: stuck).
- **Consequences:** Re-enable Cycle All. If the shroud freezes, save without idle is not enough and we fall back. PWM unchanged.

### 2026-09-14 — Restore GPU Cycle All saved static snaps
- **Status:** Superseded
- **Context:** RGB1-only `i2ctransfer` snaps left the 4090 shroud stuck on one color. This ITE does not display RGB1 until idle + static + save. The 500 ms Apply-class snap was walking the wheel (with pops).
- **Decision:** `set_snap` writes the full saved static sequence again (idle, RGB1, brightness, BGR blocks, `0x13`, `0x3F`). `GPU_MS` stays 500. No `i2ctransfer`.
- **Alternatives considered:** Keep RGB1-only (rejected: stuck). Firmware rainbow (rejected: unsynced).
- **Consequences:** Shroud should walk again. Pops from idle/save may return. PWM unchanged.

### 2026-09-14 — GPU snap is RGB1 only
- **Status:** Superseded
- **Context:** Skipping idle was not enough. Snaps still wrote BGR `0x27–0x29`, static `0x13`, and save `0x3F` every 500 ms. Those registers are for multi-color effects; Direct only uses RGB1.
- **Decision:** `snap` is one `i2ctransfer` to `0x30–0x32`. Apply still arms with idle + RGB1 + brightness + static + save, and does not write the color blocks. `GPU_MS` stays 500.
- **Alternatives considered:** Keep zone blocks on the snap (rejected: they flash). Save every snap (rejected: blink).
- **Consequences:** Re-enable Cycle All. If the jewel stops walking, RGB1 is not live without save. PWM unchanged.

### 2026-09-14 — GPU Cycle All flashes were idle + bytewise RGB
- **Status:** Superseded
- **Context:** 30 s of Cycle All showed only `cf-gpu` writing `i2c-0` `0x68`. Captured idle `0x22=0x1C`, BGR block `0x28`, static `0x13`, save `0x3F`. No second process. Idle drops static so firmware rainbow leaks; R/G/B 20 ms apart are wrong intermediate colors.
- **Decision:** After the first arm, `set_snap` stays in static: one `i2ctransfer` RGB1, zone blocks, `0x13`, save. No idle. Apply still uses the full idle sequence. `GPU_MS` stays 500.
- **Alternatives considered:** Keep idle every snap (rejected: that is the glitch). Firmware rainbow (rejected: unsynced).
- **Consequences:** Re-enable Cycle All on the debug GUI. If color stops walking, idle may still be required. PWM unchanged.

### 2026-09-14 — GPU Cycle All snaps the host wheel every 500 ms
- **Status:** Superseded
- **Context:** 1 s snaps stayed smooth. The user asked to try 500 ms.
- **Decision:** `GPU_MS` is 500. Same Apply-class `set_color` (save) from `PHASE`. Stop sleep stays 50 ms slices. No bus dump.
- **Alternatives considered:** Stay at 1 s (rejected: user asked to try 500 ms). Return to 120 ms lamp rate (rejected: freeze).
- **Consequences:** I2C+save about twice as often as 1 s. Watch for GUI hitch. PWM unchanged.

### 2026-09-14 — GPU Cycle All snaps the host wheel every 1 s
- **Status:** Superseded
- **Context:** 2 s saved static snaps were an acceptable sync workaround. The user asked to try 1 s and watch GUI load.
- **Decision:** `GPU_MS` is 1000. Same Apply-class `set_color` (save) from `PHASE`. Stop sleep stays 50 ms slices. No bus dump.
- **Alternatives considered:** Stay at 2 s (rejected: user asked to try 1 s). Return to lamp-rate I2C (rejected: freeze).
- **Consequences:** About twice as many ITE saves while Cycle All runs. Hitch, if any, shows about once per second. PWM unchanged.

### 2026-09-14 — GPU Cycle All snaps the host wheel every 2 s
- **Status:** Superseded
- **Context:** Firmware rainbow `0x08` moved the shroud but drifted from Keychron. Fast Direct I2C froze the GUI. Solid Apply (saved static) is the only host color the jewel follows.
- **Decision:** `run_gpu` writes `gpu_apply::set_color` (idle, RGB, BGR, static, save) from the same 16-bit `PHASE` as the lamps, at `GPU_MS` 2000. Sleep is 50 ms slices so Cycle All stop does not block the GUI for 2 s. Misses back off 5 s. No bus dump. No OpenRGB C++.
- **Alternatives considered:** Keep firmware rainbow (rejected: unsynced). Host-tick at 80–120 ms (rejected: freeze). Direct without save (rejected: no visible change).
- **Consequences:** Shroud is choppy but on-hue. Flash save every 2 s while Cycle All runs. PWM unchanged.

### 2026-09-14 — GPU Cycle All is firmware rainbow, not I2C polling
- **Status:** Superseded
- **Context:** Full Direct re-program every hue on NVIDIA adapter 1 hung the GUI (same GPU as the display; Cinnamon offered to kill it). Solid Apply worked because it is one saved I2C burst. Live RGB1 without save never moved the jewel. A one-shot rainbow `0x22=0x08` plus save `0x3F` completed in 178 ms with no leftover `i2cset`.
- **Decision:** Cycle All calls `gpu_apply::set_rainbow` once (idle, direction, brightness, speed, rainbow, save). HID lamps keep the host 16-bit wheel. Apply still uses static RGB + save. Never dump the bus. No OpenRGB C++.
- **Alternatives considered:** Host-tick Direct every 80 ms (rejected: freeze). Direct without save (rejected: user-visible no change). Leave GPU off Cycle All (rejected: shroud must cycle).
- **Consequences:** Shroud rainbow is on-chip, not hue-locked to Keychron. GUI must stay responsive. PWM unchanged.

### 2026-09-14 — GPU Cycle All re-programs Direct each hue
- **Status:** Superseded
- **Context:** Apply All solid on the 4090 shroud worked (full static + save `0x3F`). Cycle All stayed on that color because live frames only wrote RGB1 `0x30–0x32` after the first arm. The ITE does not follow RGB1-only after a saved static fill; OpenRGB Direct re-enters idle then static on every LED update and does not save.
- **Decision:** `set_live` always writes unknown `0x2E`, idle `0x1C`, RGB1, brightness, BGR blocks `0x27–0x29`, static `0x13`, never `0x3F`. `begin_live` clears the last RGB when Cycle All starts. 20 ms pacing. No bus dump. No OpenRGB C++.
- **Alternatives considered:** Keep RGB1-only after arm (rejected: user-visible stuck color). Save every hue (rejected: flash wear; Apply already saves). Firmware rainbow `0x08` (rejected: not the host 16-bit wheel).
- **Consequences:** Cycle All on the debug GUI should walk the jewel with Keychron. Idle between hues may flicker slightly. PWM unchanged.

### 2026-09-14 — GPU shroud ITE is NVIDIA adapter 1; never dump
- **Status:** Accepted
- **Context:** Skipping adapter 1 and requiring RGB readback left the 4090 shroud dark. Writes to `i2c-0` `0x68` do change the jewel: idle `0x1C` without finishing static `0x13` looks off. `i2cget`/`i2cdump` are garbage or wedge the adapter `EIO`. OpenRGB MSI GPUv2 Direct on NVIDIA uses port 1.
- **Decision:** Prefer NVIDIA adapter 1. Arm with unknown `0x2E=0`, idle `0x1C`, RGB1 `0x30–0x32`, brightness `0x64`, BGR blocks `0x27–0x29`, static `0x13`, save `0x3F` only on Apply. Live frames write RGB1 only (20 ms per byte), no idle, no save, no readback. Cycle All still backs off 5 s on `EIO`. No OpenRGB C++.
- **Alternatives considered:** Keep skipping adapter 1 (rejected: that is the ITE on this host). Trust `i2cget` (rejected: ITE does not read colors). Dump the bus to debug (rejected: `EIO` until the adapter recovers).
- **Consequences:** Re-enable Cycle All on the debug GUI. Never `i2cdump` GPU I2C. PWM unchanged.

### 2026-09-14 — GPU I2C adapter 1 is dummy; stop Cycle All storm
- **Status:** Superseded
- **Context:** Cycle All kept the 4090 shroud red. Live `i2cdetect` shows `0x68` only on NVIDIA **adapter 1** (`i2c-0`). Writes ACK, then `i2cget` returns `0x3f`/`0x46` and an `i2cdump` leaves the adapter `EIO`. Adapters 3–6 have no live ITE. OpenRGB detect skips NVIDIA `port_id == 1` (DDC).
- **Decision:** `skip_dummy` ignores adapter 1. Apply/live require RGB readback to match. Cycle All backs off 5 s after misses instead of `i2cset` every 120 ms. Live frames do not write save `0x3f`. No OpenRGB C++.
- **Alternatives considered:** Keep writing adapter 1 (rejected: dummy ACK, then EIO). Translate MSIGPUv2 C++ (rejected: ADR-0007).
- **Consequences:** The on-card jewel/nameplate stays dark/red until a non-DDC NVIDIA I2C map exists. Chassis AIO ARGB remains Fusion D_LED. PWM unchanged.

### 2026-09-13 — Cycle All live GPU I2C
- **Status:** Accepted
- **Context:** Cycle All skipped NVIDIA I2C because `i2cget`/`i2cset` on the lamp thread stalled the pointer (same class of extra work as Prime `0x59`). The shroud then stayed on the last Apply color.
- **Decision:** `gpu_apply::set_live` skips duplicate RGB and `try_lock`s so a slow SMBus write cannot stack. A `cf-gpu` thread (120 ms) follows the same 16-bit `PHASE` as HID lamps. After the first arm, only ITE RGB regs `0x30–0x32` are written. Apply still uses `set_color` (blocking lock).
- **Alternatives considered:** Keep GPU off Cycle All (rejected: user wants the shroud on the wheel). `i2cset` on the HID lamp thread (rejected: a 0.2 s timeout would stretch Keychron/Prime).
- **Consequences:** Re-apply Cycle All on the debug GUI. GPU util still caches 2 s. PWM unchanged.

### 2026-09-13 — Shared 400 ms hardware gauges tick
- **Status:** Accepted
- **Context:** Lighting skipped `hardware_gauges` while Cycle All ran, polled at 1 s, and omitted CPU/GPU load from the hist key, so usage gauges froze. Cooling had a separate 1 s TempsList interval that unmounted off-tab.
- **Decision:** `gaugesTick.js` polls `hardware_gauges` every 400 ms from `App.svelte` (150 samples ≈ 60 s). Cooling Temps and Lighting GaugeMeter consume the same `gauges`/`hist`. nvidia-smi stays on its 2 s cache (ADR-0017). Cycle All still skips HID/OpenRGB apply on the lighting tick.
- **Alternatives considered:** Keep Lighting poll paused during Cycle All (rejected: `/proc` + cached smi is not USB). Push nvidia-smi onto the LED frame (rejected: hitch).
- **Consequences:** Usage bars and sparks update on Lighting while rainbow runs. GPU utilization still steps at most every 2 s. PWM unchanged.

### 2026-09-13 — Keychron Direct type 0 (static), not breathing
- **Status:** Accepted
- **Context:** Cycle All walked hue on Direct but keys dimmed to black. `0xA8`/`0x08` value 1 is `PER_KEY_RGB_BREATHING`, not “per-key vs solid.”
- **Decision:** `enter_direct` sets type `0` (`PER_KEY_RGB_SOLID`) so assigned HSV stays at full value. VIA custom `0x17` still shows the per-key buffer. No RGB_SAVE.
- **Alternatives considered:** Drop Direct and return to VIA SOLID hue (rejected: loses per-key). Zero VIA speed only (rejected earlier: `0x17` can still animate).
- **Consequences:** Re-apply Cycle All on the debug GUI. PWM unchanged.

### 2026-09-13 — Cycle All Direct fill; Prime live HID
- **Status:** Accepted
- **Context:** VIA SOLID hue cannot address per-key Direct. Prime `0x62` + 50 ms + EEPROM `0x59` on the animation clock stalled Cinnamon because lighting shares the mouse USB device.
- **Decision:** Cycle All arms Keychron Direct once and `set_fill`s all 108 keys on the 120 ms lamp thread (skip unchanged HSV; no VIA GET / `drop_session`). Prime `set_live` writes `0x62` on a persistent fd and skips duplicate RGB; `0x59` save is Apply plus one shot on cycle stop. GPU I2C stays off Cycle All.
- **Alternatives considered:** Keep SOLID hue (rejected: no per-key control). Firmware rainbow (rejected: unsynced with Fusion). OpenRGB QMK flash (rejected: HUMAN flashing / GPL).
- **Consequences:** Uniform Cycle All is still 256 HSV steps on the keys. Pointer should stay smooth. PWM unchanged.

### 2026-09-13 — Cycle All 16-bit RGB fade; skip mouse/GPU I2C
- **Status:** Accepted
- **Context:** 8-bit HSV steps still looked like pops. Painting Prime (mouse HID) and NVIDIA I2C every lamp tick still made Cinnamon hitch. Fusion uniform rewrote D_LED padding on every frame.
- **Decision:** Internal hue is 16-bit (65536 steps) converted to RGB for Arena/Fusion. Keychron VIA SOLID only updates when the top 8 bits change (~12 Hz over a ~20 s lap). Lamp ticks use Fusion `soft` analog (full `uniform` every 12th). Prime and GPU I2C are not written during Cycle All. `hardware_gauges` and Lighting inventory slow/stop while `cycleOn`.
- **Alternatives considered:** 60 Hz all-lamp RGB (rejected: USB/I2C stall). Firmware Fusion color-cycle (rejected: unsynced).
- **Consequences:** Keyboard firmware can only show 256 hues; RGB devices fade with ~1–2 levels per 120 ms. Mouse/GPU shroud stay on the last color until Apply All. PWM unchanged.

### 2026-09-13 — Cycle All smooth wheel; stop USB/I2C storm
- **Status:** Accepted
- **Context:** Cycle All painted Keychron, Fusion, NVIDIA I2C, Arena, and Prime every 40 ms (and spawned five threads per frame). Prime is the mouse HID; GPU `i2cget` probed the bus every frame. The desktop pointer hitching. Wall-clock hue skipped ~50 steps whenever Fusion blocked, so colors jumped.
- **Decision:** Keychron walks hue += 1 on its own thread. Other lamps follow the same hue about every 500 ms, no per-frame thread::scope. Cached GPU bus is not probed with `i2cget` each write. SOLID hue packets skip VIA GET once effect 1 is armed. JS tick drops to 500 ms while `cycleOn`.
- **Alternatives considered:** Keep 40 ms all-lamp sync (rejected: mouse/GPU stall). Firmware Fusion color-cycle (rejected: unsynced).
- **Consequences:** Full rainbow is ~10–20 s depending on speed. Lamps lag the keyboard by up to 500 ms. PWM unchanged.

### 2026-09-13 — Cycle All SOLID hue clock
- **Status:** Accepted
- **Context:** GUI Cycle All stayed on VIA effect 1 (`#ff005d`) because Direct ticks never replaced firmware SOLID, and choosing Cycle All in the dropdown did not apply until a second click. CLI Direct GET_COLOR could move with the GUI killed, but the lamps still showed the last SOLID hue.
- **Decision:** `lighting_cycle` walks HSV hue on a wall clock and paints SOLID on Keychron plus Fusion/GPU/Arena/Prime. Apply Cycle All (select change or All Devices) starts the thread; Apply All/solid stops it. Poll VIA GET hue (id 4), not GET_COLOR.
- **Alternatives considered:** Host Direct `0x17` rainbow (rejected on this Q6 HE: lamps stay SOLID). Firmware Fusion `color-cycle` (rejected: unsynced).
- **Consequences:** Live smoke is `chromaflow rgb --cycle --seconds 3 --poll`. PWM unchanged. GET_COLOR store may stay stale while SOLID hue walks.

### 2026-09-13 — Buffer-then-broadcast lighting; GET_COLOR retry
- **Status:** Accepted
- **Context:** Cycle All looked like a solid color because each tick spawned `fusion-hid.py` and raced Keychron Direct vs SOLID. Apply All white on the first try was in-flight Direct overwriting SOLID. AIO digital packets were last, so the pump lagged.
- **Decision:** One `lighting_broadcast` paints every native lamp from the same hex under a paint mutex. Fusion HID stays open (`fusion-hid.py serve`) and writes D_LED before analog. Apply All waits for the in-flight tick (`paintGen`) then broadcasts SOLID. Keychron SOLID retries until VIA GET effect is 1; Direct Cycle All GET_COLOR-samples LED 0 and repaints once on miss. No OpenRGB C++.
- **Alternatives considered:** Firmware Fusion `color-cycle` (rejected: unsynced). Per-device Promise.all apply (rejected: VIA/Fusion still staggered).
- **Consequences:** Live smoke is `chromaflow rgb --broadcast --color RRGGBB --poll`. PWM unchanged. GPU `0x68` readback can still be dummy `0x3f`.

### 2026-09-13 — CPU AIO all Fusion headers; GPU I2C static write; host Cycle All
- **Status:** Accepted
- **Context:** Splitting CPU AIO to analog `led6`/`led7` (Z490 guess) left the pump dark. This X570S AORUS MASTER CPU cooler is LED_CPU (12V) plus D_LED ARGB, firmware IT5701 V4.0.8.0. GPU `0x68` enumerates on NVIDIA i2c-0 but SMBus readback is dummy `0x3f`. Firmware Fusion `color-cycle` was unsynced from Keychron host ticks.
- **Decision:** CPU AIO apply uses HID feature `0xCC` on all eight analog channels plus a padded D_LED fill (`led6`/`led7`), liquidctl fallback. Motherboard Fusion stays onboard `led1`/`led3`/`led4`. GPU `set_color` writes idle `0x1C`, RGB `0x30–0x32`, static `0x13`, commit `0x3F` via `i2cset` when `i2cget` sees `0x68`. Cycle All is host-ticked `fixed`/`sync` on Fusion and GPU at 500 ms; Breathing/Flash stay firmware. No OpenRGB C++.
- **Alternatives considered:** Keep firmware Cycle All on Fusion (rejected: unsynced and missed AIO). Translate MSIGPUv2 C++ (rejected: ADR-0007).
- **Consequences:** Re-apply solid and Cycle All on the debug GUI. If the pump is still dark, analog Fusion cannot drive that header (needs a captured digital stream). GPU write can succeed on the bus and still leave the shroud unchanged if the ITE ignores dummy ACKs.

### 2026-09-13 — CPU AIO row; Fusion firmware Cycle All; Combined on every lamp
- **Status:** Accepted
- **Context:** Apply All solid white left the GPU and CPU AIO green. Fusion was one liquidctl row (`sync` + led1–led8). Cycle All host-ticked analog `fixed` colors and fought firmware. Combined from All Devices / the Hardware cube skipped Fusion (CPU lane) and GPU (`hostOk`). Dual Beacon was Rainbow Beacon with a 2× offset.
- **Decision:** List Motherboard Fusion (`led1`–`led5`,`led8`) and CPU AIO (`led6`/`led7`) as separate liquidctl devices. Cycle All/Breathing/Flash on Fusion use liquidctl `color-cycle`/`pulse`/`flash` once; host ticks stay on gauges. Combined applies to every host-ok lamp. Apply All continues after a GPU I2C error. Drop Dual Beacon. GPU `0x68` is flaky on this NVIDIA adapter; no OpenRGB C++ map (ADR-0007).
- **Alternatives considered:** OpenRGB D_LED Direct (rejected: SDK off). Translate MSIGPUv2 registers (rejected: ADR-0021).
- **Consequences:** Re-apply white / Cycle All / Combined on the debug GUI. If AIO mapping is wrong, swap which Fusion row lights the pump. GPU shroud stays unpainted until `0x68` stays up.

### 2026-09-13 — Curves from 25 °C; native host effects without OpenRGB
- **Status:** Accepted
- **Context:** Preset PWM graphs started at 30 °C / 20 °C axis, so a GPU in the high 20s sat left of the polyline. After OpenRGB was off, Keychron rainbow-style effects looked broken: uniform SOLID hid per-key frames, SET_BATCH 1 could not keep 108 keys at 100 ms, and re-entering custom `0x17` every paint restarted firmware motion.
- **Decision:** Preset points and the graph axis start at 25 °C. Per-key VIA SET batches 9 (verified GET_COLOR). Stay in Direct until the next SOLID fill. Host frames include Arena’s 4 zones. Fusion stays `sync` on ticks.
- **Alternatives considered:** Keep 1-LED SET (rejected: ~1 s/frame). Firmware QMK effects without host paint (rejected: IDs on this Q6 HE are not a stable Direct).
- **Consequences:** Re-apply a spatial effect on the keyboard in the debug GUI. Watchdog still running the previous recipe until that binary is restarted.

### 2026-09-13 — Temp floor 25 °C; Keychron SOLID; Fusion sync ticks
- **Status:** Accepted
- **Context:** GPU °C regularly sits below 30 so the temp spark sat on the floor. Combined usage on the Q6 HE tracked load (GET_COLOR V stayed 255) but the keys breathed: VIA GET effect was `0x17` (the custom id we set), which this firmware animates. Fusion CPU usage never left the picker: host frames skip `liquidctl`, Lighting unmounts off-tab, and `set_fusion` ran eight CLI processes so the first hid tick often never landed.
- **Decision:** Spark/lighting temp maps are 25–90 °C. Uniform Keychron fills set rgb_matrix SOLID `0x01` + HSV and GET the effect id. Fusion host ticks use `sync` only and apply on mode change; Apply still walks `led1`–`led8`.
- **Alternatives considered:** Keep custom `0x17` and only zero speed (rejected: GET proved `0x17` is the animated mode). Keep Fusion on Lighting-only hidJobs (rejected: leaving the tab skipped the first apply).
- **Consequences:** Re-apply Combined on the keyboard and CPU on the board after the debug GUI reload. PWM watchdog is untouched.

### 2026-09-13 — Cooling 400 ms poll + Keychron 108 / Fusion led1–8
- **Status:** Accepted
- **Context:** Fan cards froze while a curve `<select>` was focused (`skipPoll` treated INPUT/SELECT as pause). Keychron Apply only changed ESC: VIA SET_TYPE Solid plus 9-LED SET packets wrote LED 0. Lighting hid the Q6 board because hidraw rows had `leds: 0` and OpenRGB overlay was skipped when 6742 was closed. Fusion `sync` alone left analog headers dark.
- **Decision:** Cooling polls every 400 ms and skips only while scrolling. Conflict detect is cached 8 s. NVIDIA RPM is stale-while-revalidate (2 s). Keychron SET is per-key type and one LED per report; layout is 108 keys from hidraw. Fusion Apply sets `led1`–`led8`. Never restart `chromaflowd`.
- **Alternatives considered:** Sub-100 ms Cooling poll (rejected: nvidia-settings hitch). Keep 9-LED SET batches (rejected: firmware kept LED 0).
- **Consequences:** Identify 100% RPM still holds. Apply All on Lighting paints the whole board plus motherboard analog zones. Addressable D_LED strips stay unmapped in Mint liquidctl.

### 2026-09-13 — 100% curve vs packaged watchdog
- **Status:** Accepted
- **Context:** FAN6/FAN7 set to `full` spun up then dropped to ~35%/27% duty. GUI `pwm_takeover` writes 100%; `chromaflowd` is still `/usr/bin/chromaflow` (03:31) which does not hold `full`, and channel `step_down=5` winds duty back every 2s.
- **Decision:** `flat_full` skips hysteresis/step and still writes 100% if the temp gauge is missing. This session’s user unit `ExecStart` is the debug `chromaflow` so the live watchdog matches the GUI. Failsafe `ExecStopPost` stays. Never restart by `systemctl stop` without a neutralized failsafe blip plan.
- **Alternatives considered:** Leave packaged watchdog (rejected: 100% cannot hold). Failsafe-to-100% (rejected, ADR-0018).
- **Consequences:** Identify headers stay at duty 255 until the curve changes. Reinstall `.deb` before relying on `/usr/bin/chromaflow`.

### 2026-09-13 — Classic IIFE for WebKit custom-protocol UI
- **Status:** Accepted
- **Context:** The GUI window painted only `:root` `#121418`. An inline classic script turned the surface brick-red, so HTML+CSS ran and ES `type="module"` did not (custom-protocol CORS). Vite 8 also put a classic IIFE in `<head>` before `#app`.
- **Decision:** Production Vite output is one deferred IIFE (`inlineDynamicImports`, no `type="module"`). `main.js` uses Svelte 5 `mount` after `DOMContentLoaded`. Keep WebKit DMA-BUF + compositing disabled on this NVIDIA host.
- **Alternatives considered:** Keep ES modules and relax CSP (rejected: WebKit still CORS-fetches modules). `new App()` constructor (rejected: Svelte 5).
- **Consequences:** Debug `chromaflow-gui` shows Cooling chrome. Installed `/usr/bin/chromaflow-gui` stays stale until the `.deb` is rebuilt.

### 2026-09-13 — Native lighting without OpenRGB (ADR-0021)
- **Status:** Accepted
- **Context:** OpenRGB respawned from GUI/`chromaflow-sdk.service`, held GPU I2C, and still failed SMBus `EIO` at `0x68`. Keychron/Arena/Prime/Fusion already had native or liquidctl paths.
- **Decision:** Do not spawn OpenRGB unless `CHROMAFLOW_OPENRGB_SDK=1`. Disable the SDK user unit. Lighting lists hidraw + Fusion + PCI GPU. Fusion apply is liquidctl only. GPU color waits on a live I2C map (no OpenRGB C++). Never restart `chromaflowd`.
- **Alternatives considered:** Keep OpenRGB as silent fallback (rejected). Copy MSIGPUv2 registers (rejected, ADR-0007).
- **Consequences:** Motherboard and USB RGB work with 6742 closed. GPU shroud RGB is listed but not painted until `0x68` is verified.

### 2026-09-13 — Native lighting ports + OpenRGB identity catalog (ADR-0020)
- **Status:** Accepted
- **Context:** MIT cannot vendor OpenRGB controllers, but the lighting gap list should be every device OpenRGB 1.0 already supports. USB color on this host should not depend on Qt/TCP for Keychron.
- **Decision:** Extract detector identity (name/bus/IDs) from the AppImage-pinned git revision into `data/openrgb-device-index.csv`. Overlay marks native/extra/sdk. `lighting_port` + Keychron VIA SET `0x0A` with GET_COLOR verify. OpenRGB sibling stays. Relicense remains off.
- **Alternatives considered:** GPL the product and copy Controllers (rejected). Dump thousands of rows into PRODUCT_GAPS.md (rejected).
- **Consequences:** Fusion/GPU stay sdk-fallback until independent captures. `CHROMAFLOW_NATIVE_RGB=0` rolls back to SDK-only.

### 2026-09-13 — OpenRGB reap + skip unchanged nvidia-settings
- **Status:** Accepted
- **Context:** OpenRGB AppImage aborted in Qt/X11 and was left as a zombie (`mem::forget`). Watchdog called `nvidia-settings -a` every ~2s at the same 20%, hitching Cinnamon.
- **Decision:** Keep a `Child` and `try_wait`; treat `/proc` state `Z` as dead; 30s backoff after a failed spawn; `libxcb-cursor0` in `.deb` Depends. GPU fan apply is skipped when percent is unchanged; nvidia query TTL is 8s. Never restart `chromaflowd` for lighting.
- **Alternatives considered:** Protocol-5 OpenRGB rescan (still deferred). NVML instead of nvidia-settings (not this slice).
- **Consequences:** Existing zombie is cleared by `systemctl --user restart chromaflow-sdk` after the new binary is installed.

### 2026-09-13 — Lighting SDK after graphical session (ADR-0019)
- **Status:** Accepted
- **Context:** After reboot, OpenRGB started at `default.target` with no `DISPLAY`, so the GPU and Keychron were missing and hidraw leftovers froze on the Lighting tab.
- **Decision:** SDK/GUI units follow `graphical-session.target` with X11 env; one `openrgb.pid` SIGTERM restart in a 45s boot window; Lighting keeps polling while research leftovers exist. PWM watchdog stays on `default.target`.
- **Alternatives considered:** OpenRGB protocol-5 rescan packet 140 (DATA layout risk). Restarting `chromaflowd` (rejected).
- **Consequences:** Existing installs need `daemon-reload` + `systemctl --user restart chromaflow-sdk` (not the watchdog) until the next `.deb`.

### 2026-09-13 — Ship with Vite 5.4 / Svelte 4 despite GHSA High on `vite`
- **Status:** Accepted
- **Context:** `/ship` local `upd audit --check` fails on GHSA-fx2h-pf6j-xcff (High, Vite `server.fs.deny` bypass on Windows ADS/8.3). Patch is Vite `6.4.3` / `7.3.5` / `8.0.16`. Desktop is Svelte 4 + `@sveltejs/vite-plugin-svelte` 3 (Vite 5). Golden Path `examples/web` already uses Vite 8.3. Svelte SSR GHSAs fix only in Svelte 5.51+.
- **Decision:** Keep desktop on Vite `^5.4.21` and Svelte 4 for this 0.1.0. Production `chromaflow-gui` embeds a Vite **build**, not the dev server. `vite.config.js` binds `127.0.0.1` only. The High is Windows-dev-server + `--host`/LAN. Track Vite 6+/Svelte 5 on the Windows port plan.
- **Alternatives considered:** Vite 6.4.3 + plugin/Svelte 5 rewrite in this ship (too large, untested on Mint). `--no-fail` on audit (disables the gate).
- **Consequences:** Local `pre-release-gate.sh --local` stays red on `upd audit --check` until the Windows/Vite major. `npm audit --audit-level=high` in Golden Path is clean. Do not expose `vite --host` on Windows until that bump.

### 2026-09-13 — Locked app mark and color taskbar icon
- **Status:** Accepted
- **Context:** The Golden Path red triangle was still the `.deb` / Tauri icon. The chosen mark is a navy rounded square, full hue ring, and gold three-blade fan with motion trails, plus a photorealistic glass-floor hero.
- **Decision:** PNG is canonical (`branding/assets/chromaflow-icon.png` and `chromaflow-icon-hero-glass.png`). Linux ships hicolor 16–512 plus SVG. The GUI sets a color window icon at runtime. `icon.ico` (16–256, 32-bpp PNG frames) is the Windows taskbar/Start icon for a future packager. README hero is the glass render; Demo is the 2D mark.
- **Alternatives considered:** Pure SVG trails (cannot match the locked raster). Monochrome/symbolic panel icon (user asked for color). Skip `.ico` until a Windows PWM port exists (then the first Windows build would still ship the triangle).
- **Consequences:** `tauri` enables `image-png` / `image-ico`. Windows PWM/hwmon is still out of scope. GitHub social PNG stays uncommitted (over the 500 KB hygiene cap); SVG social preview is the tracked fallback.

### 2026-09-13 — Single-instance installed GUI
- **Status:** Accepted
- **Context:** Multiple `tauri dev` / `cargo run` windows stacked. Product launch should be the Mint `.deb`, one window.
- **Decision:** `chromaflow-gui` binds `$XDG_RUNTIME_DIR/chromaflow-gui.sock`. A second process connects, the first focuses `main`, the second exits 0. `.desktop` sets `SingleMainWindow=true`. Install via `chromaflow_*.deb`.
- **Alternatives considered:** `tauri-plugin-single-instance` (extra crate + capability). Kill-other-PIDs (racy, can hit the watchdog).
- **Consequences:** Stale sockets after a crash are removed when connect fails. Dev `tauri dev` and the installed binary share the same socket name, so only one GUI runs.

### 2026-09-13 — Board Auto-calibrate, short equal cards
- **Status:** Accepted
- **Context:** Per-card Tune and Auto-calibrate made fan/pump tiles tall and duplicated the same sweep. Temp cards no longer matched after Tune was collapsed.
- **Decision:** One Cooling toolbar Auto-calibrate takes over writable ITE headers and sweeps fans then pumps (existing `pwm_calibrate`, 20–100%, never silent 0%). NVIDIA GPU fans stay skipped. Fan/pump/temp tiles share `min-height: 15.5rem`; curve cards stay graph-sized.
- **Alternatives considered:** Keep per-card Tune behind a disclosure (Sprint 24) — still added height. A new bulk Rust `sweep_all` — unnecessary; sequential IPC matches Fan Control’s one-header-at-a-time dwell.
- **Consequences:** Recipe min%/hysteresis stay at schema defaults unless loaded from `curves.json`. Live board sweep is HUMAN (fans spin through 100%).

### 2026-09-13 — Curve fit + unmount idle tabs
- **Status:** Accepted
- **Context:** Curve °C/% labels sat in HTML overlays outside the SVG and were clipped by `contain: content` / `content-visibility` plus 33rem tiles. Cooling still felt laggy because Lighting stayed mounted (`display: none`) with the 108-key board, and the 15 s inventory poll called `ensure_sdk` + full OpenRGB/hid/liquidctl while the user was on Cooling.
- **Decision:** Draw axis and point labels inside a padded SVG (`xMidYMid meet`, 7.25rem). Unmount Lighting/Profiles/Support off-tab. Cooling `inventory({ light: false })` uses `collect_cooling` (hwmon + GPU fans + cooling gaps only). Collapse fan Tune fields. Temps poll `hardware_gauges` at 2 s while Cooling is mounted.
- **Alternatives considered:** Keep Lighting mounted and pause ticks (Sprint 23) — still paid for a hidden 108-key DOM. Equal-height 33rem tiles — caused crop and huge paint. Electron/iced rewrite — does not remove sysfs/OpenRGB IPC.
- **Consequences:** Host lighting animations stay frozen while Cooling is open (same as Sprint 23) and the Lighting tree is rebuilt on return. Cooling RPM poll no longer starts OpenRGB.

### 2026-09-12 — Cooling scroll vs hidden Lighting ticks
- **Status:** Accepted
- **Context:** Cooling still juddered while scrolling: Lighting stays mounted, so a 10 Hz OpenRGB/HID tick plus 1 s gauge SVG updates ran during Cooling scroll. Inventory updates also rebuilt the 108-key Lighting tree.
- **Decision:** Pause OpenRGB/HID and Lighting inventory while `ui.page !== Lighting` or `ui.scrolling`. Freeze Lighting's inventory prop on other tabs. Skip Temps hist while scrolling. `content-visibility: auto` on cooling tiles. Keep Tauri+Svelte (WebKit) — rewriting to Electron/iced does not remove sysfs/OpenRGB IPC.
- **Alternatives considered:** Unmount Lighting (would stop host effects even when you want them after returning). Electron (heavier). Native GTK rewrite (same WebKit paint if we keep HTML cards).
- **Consequences:** Host lighting animations freeze while the Cooling tab is open, then resume on Lighting. Clone copies a preset; Quiet/Balanced/Performance stay locked.

### 2026-09-12 — One-password Install all + Cooling board polish
- **Status:** Accepted
- **Context:** Install all was already one helper script, but polkit `auth_admin` re-prompted. Kernel extras cannot ship as `.ko`. Cooling 1 s gauge/`pwm_tick` rebuilt every fan card and stuttered while scrolling. Mix checkboxes sat under temp cards. Curves were presets only. Temp graphs were short and had no scale.
- **Decision:** `auth_admin_keep` on install-support. Ship `60-chromaflow.rules` in the `.deb` (`/usr/lib/udev/rules.d`) and Recommends `liquidctl`, `i2c-tools`, `lm-sensors`. Never vendor `.ko`. Move 1 s hist into Temps; pass stable mix ids to fan dropdowns; stop GUI `pwm_tick`; skip inventory poll while Cooling scrolls. Equal-height temp/fan tiles; spark wireframe 0–100% / 30–90 °C with a vertical bar every 15 s. Temps **+** for mixes; Curves **+** for named custom points (`points_for` in the watchdog).
- **Alternatives considered:** Nested pkexec per extra (rejected: extra passwords). Bundling frankcrawford `it87.ko` (rejected: AGENTS.md). GUI `pwm_tick` alongside `chromaflowd` (rejected: double `nvidia-settings`).
- **Consequences:** Deb install covers udev/helpers; remaining extras still need one Install all password. Custom curves persist in `curves.json` and apply on the next daemon tick.

### 2026-09-12 — First Balanced scheme + bulletproof extras
- **Status:** Accepted
- **Context:** Kernel extra Install re-ran DKMS/modprobe on present items and failed loudly. liquidctl was optional apt only. Temp cards had no usage history. Page chrome scrolled away. Live PWM apply refused `/sys/class/hwmon` device symlinks as path escapes. User confirmed take-over.
- **Decision:** Skip extras already present. Extra `liquidctl` (apt). Temp cards show usage % plus 60 s usage (green→red) and temp (blue→red, 30–90 °C) graphs. Lock top bar and tab rail. Resolve PWM paths inside the canonical chip dir. `chromaflow cooling --takeover` writes Balanced (min 20%), calibrates spinning headers, enables `chromaflowd`.
- **Alternatives considered:** CoolerControl apt for liquidctl (rejected). Sweep 0 RPM headers in bulk take-over (skipped; per-card calibrate still can). NVML (rejected: closed SDK).
- **Consequences:** This host: all extras present including liquidctl (Fusion only). Watchdog owns 8 ITE PWM + 2 NVIDIA fans. `chromaflowd` is the user unit; failsafe is `pwm*_enable=2` when that unit stops.

### 2026-09-12 — Support extras stay green; Cooling temp cards
- **Status:** Accepted
- **Context:** PWM sysfs Install dumped pwm-acl stdout + JSON into Support. Uncontrolled devices copy was long. Cooling temp dropdowns overflowed cards; AIO checkbox only on some rows. CoolerControl uses hwmon, liquidctl, NVML, nvidia-settings/smi.
- **Decision:** Parse apply JSON after helper banners; hide extra errors when present. Temps are CPU/GPU/RAM/Disk/Combined/mix cards. AIO checkbox on every control card. Optional apt `liquidctl`. No NVML. No CoolerControl apt.
- **Alternatives considered:** NVML GPU fans (rejected: closed SDK). Root sensors-detect `/dev/port` (rejected: GUI never root).
- **Consequences:** USB CPU AIO fans stay undetected until `liquidctl` is installed and lists speeds.

### 2026-09-12 — Support extra it87-dkms first
- **Status:** Accepted
- **Context:** In-tree `it87` binds IT87952E only on this X570S. Host `it87-dkms` (frankcrawford, already in CoolerControl apt) binds IT8689. Support listed in-tree ITE as present and had no DKMS row. PWM sysfs stayed `644 root:root`.
- **Decision:** Extra kernel support lists `it87-dkms` first. Present is live `srcversion` vs `/lib/modules/$(uname -r)/updates/dkms/it87.ko*`. Apply `apt-get install it87-dkms` only if the package is already in apt; write `chromaflow-it87.conf`; reload `it87`. Never add CoolerControl apt. Never `modprobe it87-dkms`. Smoke `pwm_acl` on the pinned helper so plugdev gets `0660` on `pwm*` / `pwm*_enable`.
- **Alternatives considered:** Treat YAML `it8689` as a `.ko` (rejected). Auto-add CoolerControl apt (rejected: competitor). Count any loaded `it87` as DKMS (rejected: hides in-tree).
- **Consequences:** Dual-ITE hosts still need a frankcrawford package in apt. ChromaFlow does not vendor `.ko`. Helper must ship `chromaflow_it87.py` and `pwm-acl.sh`.

### 2026-09-12 — Detect NVIDIA fans + dual-ITE gap
- **Status:** Accepted
- **Context:** This host has two case fans, a 3-fan CPU AIO, a 2-fan GPU hybrid AIO, and a 4090 card fan. In-tree `it87` only binds IT87952E (three headers). `nvidia-settings` already lists two GPU fans at idle 0%.
- **Decision:** Inventory `gpu_fans` from `nvidia-settings` (not NVML, not sysfs `pwm*`). Default ITE names FAN4/FAN5_PUMP/FAN6_PUMP. AIO unit take-over shares one recipe across pump headers + GPU fans. Gap `ite_primary_missing` is honest: ChromaFlow does not ship frankcrawford it87 DKMS.
- **Alternatives considered:** Userspace ISA Super I/O (rejected: root + reimplementing it87). `modprobe it8689` from YAML (rejected: not a real `.ko`). Invent RPM for 0-tach headers (rejected).
- **Consequences:** Five software controls on this machine when NVIDIA query works. Extra SYS_FAN/CPU_FAN stay missing until HUMAN DKMS. GPU take-over may need Coolbits.

### 2026-09-12 — Sprint 17 Fan Control cooling board
- **Status:** Accepted
- **Context:** Cooling was one Controls list plus a display curve. This host has three ITE PWM/tach channels, Lighting CPU/GPU gauges, and no sysfs pump node.
- **Decision:** Schema 2 recipes (control + named Quiet/Balanced/Performance + temp id, including mix and Lighting gauges). Names/mixes/calibration persist in `curves.json` plus `cooling.json`. Auto-calibrate sweeps duty 20–100% (never silent 0%) and pauses the watchdog via a lock file. Public Fan Control docs only; no vendored source.
- **Alternatives considered:** Voltage-style calibration (rejected: Super I/O is duty 0–255). Invent RPM for 0-tach headers (rejected: empty table). File `.sensor` mixes (deferred: plugin-style).
- **Consequences:** Vite still cannot write sysfs. Live RPM/calibrate smoke stays HUMAN. Time-average mix is the mean of sources this sprint, not a rolling window.

### 2026-09-12 — Live cooling: conflicts, PWM ACL, chromaflowd
- **Status:** Accepted
- **Context:** Cooling listed leftover `fancontrol`/`coolercontrold` unit files while Support showed none. Duty also died when the GUI closed because failsafe always ran.
- **Decision:** Conflicts are live `systemctl is-active`/`is-enabled` or `pidof` only. Support extra `pwm_acl` chmods Super I/O `pwm*` to plugdev 0660. Cooling take-over `enable --now`s user `chromaflowd`; GUI close skips failsafe while that unit is active. `/build` will not write live PWM.
- **Alternatives considered:** Treat leftover `.service` files as conflicts (rejected: blocks take-over after purge). Auto-run `chromaflow daemon --watchdog` against live `/sys` from HUMAN automation (rejected: needs confirm + RPM check).
- **Consequences:** Vite still cannot write sysfs. Live RPM smoke stays HUMAN. Failsafe still restores `pwm*_enable=2` when the daemon is not running.

### 2026-09-12 — PWM watchdog and firmware failsafe (ADR-0018)
- **Status:** Accepted
- **Context:** ADR-0010 blocked all duty writes. User asked for the watchdog + failsafe so Cooling can take PWM, and for Report not to be spam-clicked.
- **Decision:** `chromaflow daemon --watchdog` and GUI `pwm_takeover` write `pwm*_enable=1` and duty after confirm. Failsafe (GUI close, `ExecStopPost`, `--failsafe`) writes enable 2 for owned `hwmonN/pwmN` only. 0% needs a second confirm. Conflicts still refuse. Device Report confirms, then dims that VID:PID.
- **Alternatives considered:** Root `chromaflow` CLI (rejected: `refuse_if_root`). Failsafe to 100% duty (rejected: firmware enable 2).
- **Consequences:** Take-over stays disabled while fancontrol/coolercontrold run. Vite preview cannot write sysfs. Tests use a fake hwmon tree.

### 2026-09-12 — Live Support extras + competitor uninstall smoke
- **Status:** Accepted
- **Context:** Live Install all returned `ok: false` with empty `errors`: this kernel has no `linux-modules-extra-$(uname -r)` apt package, and `nct6775` fails with “No such device” on ITE. Cooling listed `coolercontrold` from a leftover `systemctl mask` (`symlink → /dev/null`) that Uninstall did not see. `fancontrol` was the only real competitor package.
- **Decision:** Treat ITE/Nuvoton Super I/O as one sibling (like RGB I2C). `linux-modules-extra` is present when `modinfo nct6775-i2c` works. Ignore masked units in inventory conflicts. Live `fancontrol` purge succeeded; OpenRGB untouched.
- **Alternatives considered:** Fail Install all until the extra metapackage exists (rejected: modules are already in-tree). Unmask leftover CoolerControl (rejected: already deinstalled).
- **Consequences:** Support shows notes for missing apt names. Still no PWM.

### 2026-09-12 — Support extra-kernel one-stop (nct6775-i2c + fans)
- **Status:** Accepted
- **Context:** Support Install for `i2c-nct6775` always failed. There is no kernel module by that name; in-tree is `nct6775-i2c`. This host already has `i2c-nvidia-gpu` loaded. Fan hwmon (`it87`, `nct6775`, `k10temp`, SMBus, DIMM SPD) was only in YAML `would_load`, not the Support extras checklist.
- **Decision:** Map extra id `i2c-nct6775` → `modprobe nct6775-i2c`. If either RGB I2C module is in `/sys/module`, both extras are present. Checklist adds `it87`, `nct6775`, `k10temp`, `i2c-piix4`, `jc42`, `spd5118`. Do not treat Super I/O siblings as interchangeable. Do not set `acpi_enforce_resources=lax`.
- **Alternatives considered:** Keep probing `i2c-nct6775` (rejected: module does not exist). Mark every extra green when NVIDIA I2C is up (rejected: `linux-modules-extra` and DIMM SPD stay honest). Auto-apply `acpi_enforce_resources=lax` for RAM °C (rejected: reboot + `[HUMAN]`).
- **Consequences:** Pinned helper must be rebuilt for live `--only it87`. `nct6775` may stay empty on ITE boards. Still no PWM.

### 2026-09-12 — Support uninstall of competing fan/RGB daemons
- **Status:** Accepted
- **Context:** This host runs `fancontrol` + `coolercontrold`. Users need a Support action to remove those and other fan/RGB stacks that fight ChromaFlow, without touching OpenRGB (lighting path) or NVIDIA drivers.
- **Decision:** Allowlisted `manage-competitors.sh` via polkit `org.chromaflow.remove-competitors`; Support shows detected names, confirms, then `systemctl disable --now` + `apt-get remove --purge`. Extra names: thinkfan, NBFC, OpenRazer/polychromatic, ckb-next.
- **Alternatives considered:** Stop-only (kept as polkit `--stop-apply` but UI is uninstall). Uninstall distro OpenRGB (rejected: ADR-0013 lighting engine). Generic `apt purge` from UI strings (rejected: allowlist only).
- **Consequences:** Needs the helper/deb that installs `/usr/libexec/chromaflow/manage-competitors.sh`. PWM still not written.

### 2026-09-12 — Persist last tab and per-device host effects
- **Status:** Accepted
- **Context:** Returning to the app (or leaving Lighting) always showed Cooling and Solid Color. `lastMode` lived only in memory; Lighting unmounted on tab change so the host tick stopped and lamps fell back to firmware Direct.
- **Decision:** Separate `session.json` (not `window.json`) plus `localStorage`; keep pages mounted with `.tab-hidden`; EffectList `selected` follows applied `d.mode` without snapping while the user is choosing.
- **Alternatives considered:** Mixing session maps into window-geom resize saves (rejected: a resize would wipe effects). Unmount Lighting and only restore dropdown (rejected: lamps would still go solid).
- **Consequences:** Support dry-run still runs at startup. Host effects keep USB traffic while another tab is visible.

### 2026-09-12 — Per-metric gauges, nvidia-smi GPU, Lighting poll skip
- **Status:** Accepted
- **Context:** Cinnamon still hitching while the app was open because Lighting rebuilt OpenRGB LED arrays on a 4 s inventory poll and stamped identical host frames. GPU °C was missing (no nvidia hwmon). User wanted separate CPU/GPU/combined/RAM/disk graphs and device routing.
- **Decision:** Inventory poll 15 s and skipped on Lighting; skip identical `lastPaint`; five GaugeMeter rows; `Hardware gauges` routes Aorus=CPU, 4090 AIO=GPU, Keychron+Arena=combined; cached `nvidia-smi` (2 s) only on `hardware_gauges` (ADR-0017).
- **Alternatives considered:** Uncached smi per frame (rejected). OpenRGB Hardware Sync plugin (rejected). Strip `led_colors` from inventory JSON (deferred; poll skip is enough now).
- **Consequences:** GPU °C can lag 2 s. Lighting inventory is stale until the user leaves the tab. Still no PWM.

### 2026-09-12 — Calm host USB after OS jitter
- **Status:** Accepted
- **Context:** 33 Hz `UPDATE_LEDS` on the Keychron (same USB as typing) plus a 144 Hz Lighting `requestAnimationFrame` loop made the whole desktop hitch. Gauges also spawned `df` and walked every hwmon PWM node at 2 Hz.
- **Decision:** Motion effects at 10 Hz; gauges/solid skip unchanged frames (2 Hz max); Lighting idle uses a 1 s timer; `hardware_gauges` uses `scan_temps` and a 30 s disk cache; push IPC returns `[]` so WebKit does not parse 177 LED hexes per frame. Live CPU/GPU/combined readout plus green→yellow→red spectrum and sparkline on Lighting.
- **Alternatives considered:** Keep 33 Hz for smoother chevron (rejected; OS jitter). nvidia-smi (still rejected).
- **Consequences:** Chevron is a bit coarser. Disk used% can lag 30 s. PWM unchanged.

### 2026-09-12 — Host hardware gauges and hitching fix
- **Status:** Accepted
- **Context:** Inventory OpenRGB DATA (~790 ms / 4 s) froze host animations on keys and layout together. Users wanted Hardware Sync-style meters without the GPL plugin.
- **Decision:** 33 Hz local paint + SDK push; 15 s probe/`liquidctl` cache with try_lock; gauges from hwmon + meminfo + `df -P /` as host Direct fills (ADR-0017).
- **Alternatives considered:** OpenRGB Hardware Sync plugin (rejected). nvidia-smi (hitch). Spatial LED bar (rejected for 1-LED AIO).
- **Consequences:** PWM unchanged. Device list may lag 15 s after unplug.

### 2026-09-12 — Host-effect apply was black in-app / white on keys
- **Status:** Accepted
- **Context:** Apply effect looked like a no-op: layout went black (VIA GET_COLOR store / empty SDK DATA) while the Q6 HE stayed white. Lighting remount/HMR dropped `lastMode`, so `lighting_sync([])` never sent `UPDATE_LEDS`. Chromatic frames also used the live LED hex, so black/white pickers collapsed the rainbow.
- **Decision:** Persist `lastMode`/`lastColor` on `ui`, kill stale rAF with `tickId`, stamp written host pixels onto the preview, skip SDK DATA while pushing, and force sat/value for chromatic kinds when the picker is near black or white.
- **Alternatives considered:** Trust HID overlay during host frames (rejected; it is the stored color). Keep 80 ms preview I/O (too tight for 108 LEDs plus pull).
- **Consequences:** Layout matches the frame just written. PWM unchanged. Live smoke: 16 Rainbow Moving Chevron frames at speed 32 then 255 on Keychron Q6 HE via `SET_CUSTOM` + `UPDATE_LEDS`.

### 2026-09-12 — Host Direct effects for every device
- **Status:** Accepted
- **Context:** Firmware effects do not dump per-LED animation RAM. Keychron GET_COLOR is the stored color. A UI rainbow did not match QMK math. Apply effect skipped devices that lacked that firmware mode.
- **Decision:** Host-render QMK RGB-matrix formulas and `UPDATE_LEDS` every OpenRGB controller (ADR-0016). Same `HOST_EFFECTS` catalog on all devices. Skip HID overlay while frames stream. Arena/Prime/liquidctl use LED 0 at ~30 Hz.
- **Alternatives considered:** Firmware `UPDATE_MODE` (skipped devices, no LED truth). Keychron-only stream (rejected).
- **Consequences:** Apply color stops the engine (Direct). PWM unchanged.

### 2026-09-12 — Keychron LED HID readback at display refresh
- **Status:** Accepted
- **Context:** Simulated Rainbow Wave on the layout did not match the keyboard. OpenRGB DATA is the last Direct buffer. Q6 HE VIA GET_COLOR (`0xA8`/`0x09`) returns real per-key HSV (108 LEDs, 9 per report) but not firmware animation RAM.
- **Decision:** Poll every Keychron LED on hidraw `0xFF60`. While the board is expanded, Direct-stream a host frame and paint the HID readback. GUI uses `requestAnimationFrame` (monitor Hz, including 144). ADR-0015.
- **Alternatives considered:** Faster SDK polling (stale). Per-LED HID at 144 Hz (USB cannot). Keep UI simulation (rejected).
- **Consequences:** Expanding the Keychron card takes Direct. PWM unchanged.

### 2026-09-12 — Firmware effect handshake + LED layout simulation
- **Status:** Accepted
- **Context:** Apply effect on Keychron Q6 HE needed two clicks (first flash white, then Rainbow Wave). The expanded LED layout stayed solid green while the keyboard ran rainbow. OpenRGB `REQUEST_CONTROLLER_DATA` reports the last Direct buffer, not the firmware animation.
- **Decision:** Drop the preview TCP session before apply. Send `UPDATE_MODE` twice with a DATA re-read. Do not paint mode colors onto rainbow/wave/cycle blobs. ColorFrame includes `active_mode` + mode name. The layout simulates the announced effect unless polled pixels already vary; Direct/Custom still use live `led_colors`.
- **Alternatives considered:** Faster LED polling (rejected: SDK pixels are stale). Client-side OpenRGB Direct streaming of firmware effects (rejected: fights the device animation).
- **Consequences:** Layout is a faithful preview of named firmware modes, not a camera of the keyboard. PWM unchanged.

### 2026-09-12 — Picker wrap + window size + Prime Neo HID
- **Status:** Accepted
- **Context:** Lighting controls were one tall column; the window reset to 1280×800 every launch. Prime Neo (`1038:1856`) is controllable in SteelSeries GG; OpenRGB does not list it.
- **Decision:** Color wheel, sliders, hex/swatches, and effect are separate `.picker-card` objects in a wrapping flex row. Persist inner size (and maximized) to `window.json`. Prime Neo vendor hidraw uses rivalcfg-documented output `0x62` plus save `0x59` (ADR-0014); no rivalcfg spawn.
- **Alternatives considered:** CSS `display:contents` (inheritance risk). Tauri window plugin (unnecessary). Vendoring rivalcfg (GPL; rejected).
- **Consequences:** Remaining research HID is still no-Apply. PWM unchanged.

### 2026-09-12 — Bundled OpenRGB engine + Mint .deb
- **Status:** Accepted
- **Context:** Lighting needs a localhost SDK after uninstalling the OpenRGB desktop app. ChromaFlow cannot vendor GPL C++. Users install via Cinnamon menu, not an AppImage.
- **Decision:** One `chromaflow_*.deb` with `.desktop`. Sibling OpenRGB AppImage (ADR-0013) spawned unprivileged on 127.0.0.1. Research HID list (cap 32). Support GitHub device form; serial opt-in.
- **Alternatives considered:** GUI AppImage (withdrawn). In-process OpenRGB (ADR-0007). OpenRGB Start-menu entry (rejected).
- **Consequences:** Tests cover hash reject, `CHROMAFLOW_NO_SPAWN`, no OpenRGB.desktop. User systemd unit is a follow-up. PWM unchanged.

### 2026-09-12 — OpenRGB modes + per-LED matrix + Kelvin
- **Status:** Accepted
- **Context:** Lighting needed firmware effects and a keyboard layout like OpenRGB, plus CCT for matching room lamps. Slider labels were smaller than page body text.
- **Decision:** Parse protocol-0 `modes[]` and zone `matrix_map` from SDK DATA. `UPDATE_MODE` (1101) and `UPDATE_SINGLE_LED` (1052) on localhost. UI lists that device’s modes (OpenRGB Common Modes names, not a hardcoded enum). Kelvin slider uses Tanner CCT→RGB. Slider row font inherits body size.
- **Alternatives considered:** Hardcoded global effect list (rejected: each controller reports its own modes). Vendored OpenRGB key maps (rejected: ADR-0007). Protocol 3+ brightness fields (rejected: stay on protocol 0).
- **Consequences:** Arena HID has no SDK modes. Direct/Custom still use `SET_CUSTOM` + `UPDATE_LEDS`. PWM unchanged.

### 2026-09-11 — One-click Support apply via pkexec
- **Status:** Accepted
- **Context:** Detection support was dry-run in the GUI. The user needs one click for linux-modules-extra, i2c-dev, Gigabyte WMI, udev, groups, and experimental RGB I2C.
- **Decision:** Tauri `support_apply` and `chromaflow support --apply` run `pkexec` of the pinned helper. `--advanced` is a second polkit action. Optional apt (`openrgb`) failures are warnings. `nouveau` is never loaded.
- **Alternatives considered:** GUI as root (rejected). In-process apt from Tauri (rejected: pinned helper only).
- **Consequences:** Password dialog is the Cinnamon polkit agent. Log out after group/udev changes. PWM still off.

### 2026-09-11 — Arena 7 HID + Fusion D_LED radiator headers
- **Status:** Accepted
- **Context:** OpenRGB does not list Arena 7 (`1038:1a00`). Fusion `D_LED1`/`D_LED2` were size 0 so chassis radiator ARGB never received `UPDATE_LEDS`.
- **Decision:** Arena 7 vendor hidraw output report `0x06` (ADR-0012). Fusion apply resizes D_LED zones to 32 and sends `UPDATE_ZONE_LEDS`. Prime Neo stays display-only.
- **Alternatives considered:** OpenRGB Arena controller (not in git2478; do not vendor C++). GPU I2C extra LEDs (card already works; radiator is motherboard ARGB).
- **Consequences:** Apply All paints speakers + D_LED headers. Wrong header length is a follow-up if 32 is too short/long.

### 2026-09-11 — UPDATE_LEDS data_size must equal pkt_size
- **Status:** Accepted
- **Context:** OpenRGB git2478 logged `UpdateLEDs packet has invalid size` (10/22/438). ChromaFlow set `data_size = 2+4*n` (excluding the u32 field). The server requires `header.pkt_size == first u32`.
- **Decision:** `data_size` is `body.len()` (`4+2+4*n`). TCP close is write-only FIN. Aorus/Fusion also runs `liquidctl` after the SDK write on this 5702.
- **Alternatives considered:** `data_size == 0` protocol-≤4 workaround (rejected: this server is newer). OpenRGB CLI as the only writer (rejected: keep SDK packets).
- **Consequences:** Apply no longer reports success on a dropped packet. Inventory polls still log `recv_select failed receiving magic` on disconnect.

### 2026-09-11 — Localhost OpenRGB SDK color writes (ADR-0011)
- **Status:** Accepted
- **Context:** After per-VID udev, hidraw is `0660` plugdev. Native OpenRGB AppImage on `127.0.0.1:6742` lists Fusion (`X570S AORUS MASTER`), the 4090 AIO, and Keychron Q6 HE. Distro apt has no `openrgb` package.
- **Decision:** Lighting may send documented SDK packets (50 / 1100 / 1050) with a 200ms timeout. Fusion fallback is `liquidctl` subprocess only. No hidraw SET_REPORT, no PWM, no Flatpak server as the primary path.
- **Alternatives considered:** crates.io OpenRGB client (rejected: copyleft stop-and-ask). Direct hidraw for Arena 7 / Prime (deferred).
- **Consequences:** SteelSeries stays “No Linux backend”. Keychron can use SDK when listed, plus VIA copy. PWM remains ADR-0010.

### 2026-09-11 — USB/ARGB inventory is sysfs names, not OpenRGB control
- **Status:** Accepted
- **Context:** OpenRGB SDK on this Gigabyte X570S listed only the MSI 4090 Suprim Liquid X. USB hidraw is root-only; liquidctl sees RGB Fusion 2.0; Keychron Q6 HE, SteelSeries mouse/Arena 7, and ITE 048d:5702 are present.
- **Decision:** Inventory adds `hid_rgb` + `liquidctl_devices` from sysfs uevent/liquidctl list (no hidraw open). Lighting prompts Install detection support with Advanced experimental I2C RGB (`i2c-nct6775`, `i2c-nvidia-gpu`; never `nouveau`). `gigabyte_wmi` is a safe DMI match. Apply may install `linux-modules-extra`. No LED or PWM writes.
- **Alternatives considered:** Claim OpenRGB control of USB devices now (rejected: hidraw 600 root). Auto-load `nouveau` with Advanced (rejected: proprietary NVIDIA). Widen DMI matching so probe strings become modprobe names (forbidden).
- **Consequences:** Users see motherboard/keyboard/mouse/speakers/radiator candidates and a dry-run module list. Live pkexec apply remains HUMAN.

### 2026-09-11 — Fan Control Home chrome; PWM take-over declined
- **Status:** Accepted
- **Context:** This host runs `coolercontrold` and `fancontrol`; `it87952` PWM is writable. User asked for Fan Control-like UX and live HUMAN items.
- **Decision:** Cooling is a navy Controls + Curves card grid (yellow graph points, disabled take-over toggle). Do **not** take PWM while those daemons run. OpenRGB `--server` on `127.0.0.1:6742` is live. Merge Dependabot #1; close #3/#4. Helper `.deb` apply stays HUMAN until pkexec installs the pinned path.
- **Alternatives considered:** CoolerControl-style left sensor tree (rejected: not Fan Control). Enable PWM take-over now (rejected: conflicts + ADR-0010). Pull GitHub main Svelte 5 bump into this tree (rejected: would break Svelte 4 UI).
- **Consequences:** Native `chromaflow-gui` and Vite smoke the card UI read-only. Next HUMAN is `pkexec dpkg -i` of `target/deb/chromaflow-helper_0.1.0_all.deb`.

### 2026-09-11 — chromaflowd is watch-only until HUMAN take-over
- **Status:** Accepted
- **Context:** This host has writable `it87952` PWM and live `fancontrol` + `coolercontrold`.
- **Decision:** Ship `chromaflow daemon --dry-run`, systemd unit, and `ExecStopPost` that **logs** firmware `pwm*_enable=2` and never writes sysfs. Silent 0% remains forbidden.
- **Alternatives considered:** Write `pwm*_enable` now (rejected: conflicts + no HUMAN confirm). Skip the unit until apply exists (rejected: G-SAFE needs the failsafe contract in-tree).
- **Consequences:** PWM writes stay absent. HUMAN must confirm take-over before any enable/duty sysfs write.

### 2026-09-11 — Support dry-run is machine-specific; profiles are names only
- **Status:** Accepted
- **Context:** Full allowlist dump was not this-machine detection. Profiles were placeholder copy.
- **Decision:** Select YAML rows by lspci/DMI/lsusb, already-loaded modules, or `i2c-dev`. Emit modules-load.d **plan** with `apply: false`. Store `~/.config/chromaflow/profiles.json` schema 1 (`name`, `curve_set`, `rgb` tokens). No PWM apply.
- **Alternatives considered:** Keep dumping the whole allowlist (rejected: G-DET). Write `/etc/modules-load.d` from the GUI (rejected: polkit `--apply` only after `.deb`).
- **Consequences:** Tests must set `CHROMAFLOW_LOADED_MODULES`. Live pkexec `--apply` stays HUMAN until the pinned helper exists.

### 2026-09-11 — Product launch is a native window, not a browser
- **Status:** Accepted
- **Context:** Original brief is a standalone Mint app (Fan Control + OpenRGB UX). First-run opened Vite preview in Cursor/Chrome because this host lacks WebKit/GTK **dev** packages.
- **Decision:** Keep Tauri 2 (ADR-0006). Treat `vite preview` as developer-only. Gap catalog is `docs/PRODUCT_GAPS.md`; BUILD_PLAN Sprint 2+ is the native window, then cooling/lighting UX, then polkit `.deb` and `chromaflowd`.
- **Alternatives considered:** Rewrite in Avalonia/Qt to “look more native” (rejected: no Fan Control source; ADR-0006). Ship the PWA as the app (rejected: not a Cinnamon `.desktop` product).
- **Consequences:** `[HUMAN]` apt of `libwebkit2gtk-4.1-dev` and GTK **dev** is required before `chromaflow-gui` runs on this machine. PWM remains forbidden until Sprint 6.

### 2026-09-11 — HUMAN rows automated on Mint 22.3
- **Status:** Accepted
- **Context:** Five BUILD_PLAN HUMAN leftovers blocked GitHub/security/Mint smoke/`pkexec`.
- **Decision:** Handlers in `scripts/lib/human_task_chromaflow.py` create `edwardlthompson/chromaflow`, enable Dependabot alerts + private reporting, run inventory dry-run on this Cinnamon host, and verify `--apply` refuses without the pinned polkit path. Bookmark row is informational. Lightroom backlog is N/A (stack pruned).
- **Alternatives considered:** Leave rows for a login (rejected: `gh` is already admin-auth'd; host is Mint 22.3 Cinnamon).
- **Consequences:** Live `--apply` still needs a `.deb` at `/usr/libexec/chromaflow/install-support.sh`. First push enables the AUTO Sprint 0 sign-off.

### 2026-09-11 — Tauri 2 host is a real crate
- **Status:** Accepted
- **Context:** The first GUI stub compiled as a plain binary (`eprintln!`) so `cargo test --workspace` stayed green without WebKit, which did not match ADR-0006.
- **Decision:** `apps/desktop/src-tauri` is Tauri 2 (`tauri` + `tauri-build`, window `main`, `support_dry_run`). `default-members` stay CLI/core. CI installs `libwebkit2gtk-4.1-dev` (and `libdbus-1-dev`); Ubuntu 24.04 compile is required, 22.04 is `continue-on-error`.
- **Alternatives considered:** Keep the fake bin (rejected: not Tauri). Make every `cargo test --workspace` require GTK (rejected: CLI gates must stay runnable without WebKit).
- **Consequences:** Local `cargo build -p chromaflow-desktop` needs GTK/WebKit/dbus. Support in Vite preview still uses fixture JSON.

### 2026-09-11 — ChromaFlow Sprint 0 stack and license
- **Status:** Accepted
- **Context:** Empty workspace; brief allowed Avalonia or Tauri; GPL if OpenRGB/FanControl copied.
- **Decision:** Tauri + Rust + MIT/NOTICE; OpenRGB localhost probe; no PWM writes; polkit apply stubbed; `.deb` for helper later.
- **Alternatives considered:** Avalonia (rejected: Fan Control is proprietary). GPL-3.0 day one (rejected: no GPL sources copied; bootstrap init is MIT).
- **Consequences:** Relicense before in-process OpenRGB. AppImage cannot install polkit.

### 2026-09-11 — v1.4.0 /ship (template-gap BUILD_PLAN sync)
- **Status:** Accepted
- **Context:** Child repos needed Monday automation to list Canon/Mixed/Sacred/feature gaps on BUILD_PLAN without auto-applying Sacred overwrites.
- **Decision:** Child-only `sync-template-gaps-build-plan` on weekly-health; template keeps upgrade-sim; Sacred → `[HUMAN]`; file rows capped at 40; humans still name `/upgrade` numbers before apply.
- **Alternatives considered:** Auto-copy Canon on cron (rejected: same plan-only rule as `/upgrade`). Always run upgrade-sim on children (rejected: pruned stacks break sim).
- **Consequences:** Markers required on child BUILD_PLAN; Lightroom (#29) still HUMAN.

### 2026-09-11 — v1.3.0 /push
- **Status:** Accepted
- **Context:** Release Please #106 blocked on red CI after `chore(release): prepare v1.3.0`; instrumented Assume counted as failure; LHCI INP auditRan=0 under staticDistDir.
- **Decision:** Ship v1.3.0 after CI green; admin-merge RP #106; use TBT as lab INP proxy; vacuous-pass UnifiedPush without distributor; early-return (not Assume) for AVD-only asserts.
- **Alternatives considered:** Leave INP as error (rejected: structurally unmeasurable in navigation-mode LHCI). Keep Assume for UnifiedPush (rejected: AGP XML treats AssumptionViolatedException as failure).
- **Consequences:** Lightroom Plug-in Manager (#29) stays HUMAN; next `/allideas` batch when ready.

### 2026-09-10 — M58/M59/M60 board from allideas 1–160
- **Status:** Accepted
- **Context:** Template maintainer needed a sizeable `/build` backlog after ship CI + Espresso work; `/allideas` dump filled M58 (1–80), M59 (81–120), M60 (121–160).
- **Decision:** Keep three sequential milestones on `BUILD_PLAN.md`; archive each to `COMPLETED_TASKS.md` after `smoke-sprint --require`. Open PRs sync stays auto-managed; Release Please merge stays HUMAN.
- **Alternatives considered:** One mega-sprint (rejected: smoke/gate lock). Only Android rows (rejected: CI/docs/a11y belong on the same board).
- **Consequences:** M61 holds 161–200; do not start M61 until M60 smoke ✅.

### 2026-09-10 — Archive stale HUMAN_BACKLOG prose
- **Status:** Accepted
- **Context:** `HUMAN_BACKLOG.md` held free-form notes (declined Ollama on template; quarterly radar ownership) that are not deferred automation rows.
- **Decision:** Keep the backlog table empty unless `/build` automation fails a HUMAN/ADB row. Move standing policy into this log: optional Ollama stays in `docs/LOCAL_MODELS.md` for child repos; quarterly radar remains Monday cron (`cursor-feature-radar.sh`), not a board row.
- **Alternatives considered:** Leave prose in HUMAN_BACKLOG (rejected: confuses automation deferred items). Put Ollama back on BUILD_PLAN (rejected: maintainer declined).
- **Consequences:** Agents only append table rows via `build-backlog add`. Narrative policy lives here or in docs.

### 2026-09-10 — Pin Espresso 3.7 for Android 16 (API 36)
- **Status:** Accepted
- **Context:** Compose instrumented tests failed on API 36 with missing `InputManager.getInstance`; CI emu and phones needed a stable AndroidX Test pin.
- **Decision:** Pin `androidx.test.espresso:espresso-core:3.7.0` in Golden Path Android; add `check-espresso-android16` gate + KB-022. Keep nav Back smoke on a physical device when KVM is absent.
- **Alternatives considered:** Wait for an official test-bom (rejected: not a drop-in). Disable instrumented CI (rejected: ships broken nav).
- **Consequences:** Do not downgrade Espresso for convenience; `/push` needed for `main` CI green.

### 2026-09-10 — Defer Unreleased fold until /ship after Espresso land
- **Status:** Accepted
- **Context:** `/build` M58 row asked to fold Unreleased and cut Release Please while 180+ AGENT rows remained and commits were only local (`ahead 2`, no `/push`).
- **Decision:** Do not empty `[Unreleased]` or cut a patch until `/push` (or `/ship`) publishes the Espresso 3.7 + CI fixes. `/ship` owns fold + Release Please. After push, wait with `python3 scripts/agent-run.py wait-release-sbom -- --wait 300`.
- **Alternatives considered:** Fold now and open RP mid-sprint (rejected: incomplete board). Mark fold row blocked forever (rejected: process must stay clear).
- **Consequences:** M58 continues on This Computer; human runs `/push` then `/ship` for the Android 16 / Release checkout fixes.

### 2026-09-10 — Declutter BUILD_PLAN recurring chores
- **Status:** Accepted
- **Context:** AUTO weekly/monthly rows stayed 🔲 forever even though Monday cron already ran them, which made both boards look unfinished.
- **Decision:** Strip 🔲 chores from `BUILD_PLAN.md` and `BUILD_PLAN_TEMPLATE.md` Ongoing Maintenance. GitHub Monday cron is the owner (now includes `update-deps` dry-run, Dependabot leftover list, latest-release SBOM). Cursor Automation then Grok Bots are fallbacks only. `/ship` owns pre-release and the tag.
- **Alternatives considered:** Keep standing 🔲 AUTO rows and auto-tick them (rejected: still looks like homework). Require Grok Bots (rejected: FOSS default is Actions).
- **Consequences:** Maintainer remaining tally is ADB leftovers only. Child board is Sprint 0–2 plus optional ADB. Do not put cron chores back on either plan.

### 2026-09-10 — Maintainer schedule (GitHub cron + optional Cloud timers)
- **Status:** Accepted
- **Context:** Recurring BUILD_PLAN AUTO/AGENT rows should keep running after the maintainer leaves the Cloud Agent session.
- **Decision:** Keep GitHub Monday cron as the FOSS default (Weekly Health Check now includes `check-security-triage.sh`). Add disabled Grok Bot 4–5 and Cursor Automation crons for `/update-deps` dry-run and KB-007 review. No `git push`, no `--apply`, no live `.cursor/automations.yaml` on the FOSS path.
- **Alternatives considered:** Require Grok Bots to ship (rejected). Auto-apply dependency bumps from a Bot (rejected: needs a human). Mark release-tag HUMAN as AUTO (rejected).
- **Consequences:** AUTO weekly rows can close when Weekly Health Check is green. AGENT `/update-deps` still needs `/maintain` or a commercial timer. ADB leftovers stay on the host with the phones.

### 2026-09-10 — OpenSSF passing + Ollama declined
- **Status:** Accepted
- **Context:** Project 14564 reached 100% Passing. Maintainer does not want Ollama on this template. Cloud agent has no USB/adb for the two local phones.
- **Decision:** Keep the live Best Practices badge in README. Archive CII as done. Reject the Ollama HUMAN leftover. Leave ADB SDK + device nav smoke on the board for the host that has the phones.
- **Alternatives considered:** Require Ollama for `/ship` or local-compute (rejected). Mark ADB rows done without a device (rejected: no adb on this VM). Start Silver/Gold (rejected: coverage and two-person review are not true).
- **Consequences:** Waiting-on-a-person is ADB-only. Child repos may still follow `docs/LOCAL_MODELS.md`. Baseline-1 stays optional.

### 2026-09-10 — M57 Cursor + docs wrap
- **Status:** Accepted
- **Context:** Maintainer `/build` finished Cursor docs (Grok Bots, marketplace, skills, registry, Automations YAML, Cloud hooks, Canvas, CLI loop, Settings-only tour/coach, print-sheet audit, optional-stack gaps, ADR-0001 pick gate, living ci-gap registry). Sprint smoke first treated `/tour` `/emulator` `/adr` as file paths.
- **Decision:** Archive M57. `backtick_paths` skips slash-command tokens. Home chrome stays Settings-only. ADR-0001 stays an open child pick. `ci-ok` still must not `needs` nix.
- **Alternatives considered:** Pre-select Hexagonal on the template (rejected: child Sprint 1 pick). Make Nix a required `ci-ok` job (rejected: skipped ≠ success).
- **Consequences:** Idea sprints M51–M57 are archived (55 rows). Recurring weekly/monthly and HUMAN/ADB leftovers remain. Device nav smoke and Ollama stay backlogged.

### 2026-09-09 — M54 Catalog + Lightroom wrap
- **Status:** Accepted
- **Context:** Maintainer `/build` finished Lightroom lint/tagset/SDK playbook and catalog sync. Sprint smoke failed when a task mentioned `feature-catalog.json` as a repo-root path.
- **Decision:** Archive M54. `probe_docs` resolves bare filenames under `schemas/`, `docs/`, `examples/`, `modules/`, and `scripts/`.
- **Alternatives considered:** Fail smoke unless every backtick is a repo-relative path (rejected: board tasks name the file, not the folder).
- **Consequences:** Next AGENT row is M55 Dependabot Cargo + Go. Lightroom Plug-in Manager load stays `[HUMAN]`.

### 2026-09-09 — M53 Android distribution wrap
- **Status:** Accepted
- **Context:** Maintainer `/build` finished the Android release path (R8, reproducible APK, F-Droid, Fastlane, AntiFeatures, UnifiedPush, signing runbook).
- **Decision:** Archive M53 in `COMPLETED_TASKS.md`. UnifiedPush stays PackageManager-only (no Maven connector, no FCM). Signing uses env vars and `docs/ANDROID_SIGNING.md`; CI `android-release` stays unsigned for hash compare.
- **Alternatives considered:** Add `org.unifiedpush.android:connector` (rejected: extra Maven dep). Check a dummy keystore into examples (rejected: secrets).
- **Consequences:** Next AGENT row is M54 Lightroom Lua lint. Device UnifiedPush and real upload keys stay `[ADB]`/`[HUMAN]`.

### 2026-09-09 — Separate child BUILD_PLAN template
- **Status:** Accepted
- **Context:** The child playbook at the bottom of `BUILD_PLAN.md` did not match the slim maintainer board. Children need the same look and `/build` behavior.
- **Decision:** `BUILD_PLAN_TEMPLATE.md` is the child model (canon on `/upgrade`). `init-project` copies it onto `BUILD_PLAN.md` when the repo is not this template. Both files start with a generated remaining tally (AGENT / AUTO / HUMAN / ADB). This repo’s live board stays `BUILD_PLAN.md` only.
- **Alternatives considered:** Keep a Child Repo Playbook section (rejected: two layouts in one file). Overwrite child `BUILD_PLAN.md` on every upgrade (rejected: mixed; live rows stay).
- **Consequences:** `--lane child` on this template reads `BUILD_PLAN_TEMPLATE.md`. Tally gate: `check-build-plan-tally.sh`.

### 2026-09-09 — Sprint smoke before the next sprint
- **Status:** Accepted
- **Context:** Agents checked off BUILD_PLAN rows and moved on. The board was also too wordy to scan.
- **Decision:** `/build` wrap and `/gates` run `smoke-sprint`. Every ✅ `[AGENT]`/`[AUTO]` row in the finished sprint must pass with no errors/crashes; the report records startup time and load order. Agents do not start the next sprint until it exits 0. BUILD_PLAN stays short; detail lives in `docs/SPRINT_SMOKE.md`.
- **Alternatives considered:** Per-row `smoke-stack` alias only (rejected: that is feature-gate, not app startup). Require an emulator for every wrap (rejected: `[ADB]` leftovers; manifest + HTTP + CLI still prove load order).
- **Consequences:** `.cursor/sprint-smoke.json` is gitignored. Device TalkBack stays `[ADB]`.

### 2026-09-09 — M49: Settings-only chrome and sectioned menus
- **Status:** Accepted
- **Context:** Golden Path home chrome had Settings, About, donate, and a theme toggle; Settings used chips for exclusive enums. Material 3 app bars keep one or two trailing actions; Settings IA is grouped lists with dropdowns, not chip clouds.
- **Decision:** Home chrome is **Settings only**. Theme, About, and donate live under Settings → App info. Exclusive choices use dropdowns. Sections sort Appearance → Privacy → Data → About (then App → Support → Feedback).
- **Alternatives considered:** Quiet header donate (rejected: duplicates About). Header theme toggle plus Settings control (rejected: two places). FilterChips for theme (rejected: chips are filters).
- **Consequences:** `ThemeToggle` removed. Agents follow `docs/DESIGN_GUIDE.md` Chrome and menus. Donate walkthrough no longer allows a web header control.

### 2026-09-09 — Android runtime budget (R8 + memory limits) and optional Grok Bots
- **Status:** Accepted
- **Context:** Compose August 2026 is already on BOM `2026.08.00`. Android 17 enforces per-app memory limits. Tinder’s R8 analyzer case showed broad keep rules can leave R8 mostly idle. xAI Grok Bots are always-on commercial teammates, not a FOSS requirement.
- **Decision:** Turn Golden Path `assembleRelease` R8 on with `proguard-android-optimize.txt` and a narrow keep file; add `GoldenPathApplication` limiter/trim hooks; document Compose 1.12 don’ts and optional Grok Bot prompts. No Credential Manager or Play Services on the FOSS path.
- **Alternatives considered:** Keep minify off for readable stacks (rejected: memory + cold start). Require Grok Bots (rejected: Cloud-only default). Coil in Golden Path (rejected: no image pipeline yet).
- **Consequences:** Release builds are slower; retrace with mapping.txt; structure tests lock minify/no-largeHeap/no-broad-keep.

### 2026-09-01 — M47 wrap-up: Cline first-run and stack nav
- **Status:** Accepted
- **Context:** First-time users need autonomous help without paid keys. Golden Path Settings/About/Feedback must remember where you were and pop **one** Back without leaving the PWA or Activity.
- **Decision:** First-run is **Cline in Cursor** (GitHub sign-in, FREE model, no `OPENAI_API_KEY`). Codex stays advanced/optional (`docs/CODEX_REVIEW.md`, `/codex-review`) and is off `/tour`, `/prerelease`, and `/ship`. Navigation is a **stack** plus History API (web) and `BackHandler` (Android), persisted as `gp.nav.v1`, so menus restore and Back pops one overlay or route.
- **Alternatives considered:** Codex CLI as first-run (rejected: needs an API key). Boolean `showAbout`/`showSettings` flags (rejected: flatten About → bug; Back cannot return to About). Close calling `pop()` and `history.back()` (rejected: double-pop). Finish the Activity on first home Back (rejected: accidental exit).
- **Consequences:** Beginner path is clone → Cursor → Cline GitHub sign-in → tour prompt. Device Back smoke stays `[ADB]` until an SDK/emulator is present.

### 2026-09-01 — Golden Path Android BackHandler consumes home Back
- **Status:** Accepted
- **Context:** M47 Android wiring must pop one menu on system Back without finishing the Activity at home. This cloud agent has no ANDROID_HOME.
- **Decision:** `BackHandler(enabled = true)` always calls `NavBack.onSystemBack` which `pop()`s (prompt first, then route) and sets `finishActivity = false`. Skip the optional “press Back again to exit” snackbar. Persist via `rememberSaveable` plus SharedPreferences `gp_nav` / `gp.nav.v1`. Instrumented smoke is [ADB] until SDK is present.
- **Alternatives considered:** Snackbar 2s exit window (skipped: extra copy, first Back still must not finish). DataStore for nav (rejected: SharedPreferences matches UpdateLaunchPrefs and is readable on first composition).
- **Consequences:** Unit tests run with kotlinc+JUnit when Gradle cannot. Device/emulator smoke tracked under M43 leftovers.

### 2026-09-01 — Golden Path web Back uses history.back, popstate pops NavState
- **Status:** Accepted
- **Context:** M47 web wiring must pop one menu on browser/mouse Back without double-pop on in-app Close/Escape, and must not leave the PWA at home.
- **Decision:** Opening a panel `push()`es NavState then `history.pushState({ gp: true, depth })`. In-app Close/Escape only call `history.back()`. `window` `popstate` is the only `pop()` of NavState. At home, popstate `pushState`s `{ gp: true, depth: 0 }` again so the next Back stays on the page.
- **Alternatives considered:** Close calling `pop()` plus `history.back()` (rejected: double-pop). Close calling `pop()` and syncing history with `history.go(-1)` only when needed (rejected: two mutators). Flatten About → bug to home (rejected: stack contract).
- **Consequences:** Tests spy `history.back` as a no-op then dispatch `popstate` to prove a single pop. Persist key `gp.nav.v1`. Android BackHandler is the next AGENT row.

### 2026-08-28 — First stable v1.0.0
- **Status:** Accepted
- **Context:** Template had shipped through 0.25.0. Cloud agent `/build --lane auto` prepared `Release-As: 1.0.0` on PR #81 so Release Please would not cut 0.26.0. Upgrade-sim failed until stack-specific gates skipped after prune.
- **Decision:** Merge #81 after CI (including both upgrade-sims) was green, then merge RP #82 as v1.0.0. Keep remaining HUMAN/ADB leftovers (CII, Ollama, Android SDK) off the AGENT board.
- **Alternatives considered:** Ship 0.26.0 first (rejected: human asked for v1). Squash #81 (rejected: would drop the `Release-As` footer).
- **Consequences:** `.template-version` is 1.0.0. Child upgrades from 0.x should treat this as a major. Recurring weekly AUTO and CII/Ollama/SDK stay 🔲.

### 2026-08-28 — /build uses auto lane on the template
- **Status:** Accepted
- **Context:** Slack `/build` (bare word `build`) hardcoded `--lane child`, so the next row was Sprint 0 `init-project.sh` on this template.
- **Decision:** `/build`, `AGENTS.md`, and `docs/FOR_AGENTS.md` use `--lane auto`. On this template auto prefers Template Maintainer 🔲 AGENT rows (M46); child repos still walk the playbook. Do not run `init-project.sh` here.
- **Alternatives considered:** Run child Sprint 0 on the template (rejected: destroys template branding). Idle-exit without maintenance (rejected: M46 AGENT rows are the real next work).
- **Consequences:** M46 remaining rows stay 🔲. HUMAN leftovers remain (Ollama, DPIA, mcp.json, Dependabot, Codeowners, product smoke, Android SDK). Skipped bogus `codeql-action@vcodeql-bundle-*` tag from upd.

### 2026-08-27 — /build scoped feature-gate
- **Status:** Accepted
- **Context:** `/build` re-ran all Golden Path stacks after every AGENT row. Android weight 2 plus three RAM-capped slots made a docs-only or web-only row take several minutes.
- **Decision:** `gate_scope.py` maps git-dirty paths to `docs` (preamble only), `stacks` (touched `examples/{name}/`), or `full` (scripts/schemas/modules/shared). `/build` `/feature` `/fix` pass `--scope auto`. Autofix retries the failed stack with `--skip-preamble`. `/gates` and `/prerelease` stay full. `FEATURE_GATE_ONLY` filters the multi-stack parent.
- **Alternatives considered:** Always skip Android locally (rejected: still required when android files change). Default watch-agent-gates to auto (rejected: `/prerelease` must not silently skip stacks).
- **Consequences:** Shared script edits still run every stack. Sprint wrap-up `/gates` remains the honesty backstop.

### 2026-08-27 — /build M46 Golden Path + agent UX (rows 9–22)
- **Status:** Accepted
- **Context:** Continued `/build` on M46 after P0/schemas/Node/Python About. Android sanitizer JVM tests cannot use unmocked `org.json.JSONObject`.
- **Decision:** Robolectric for `SanitizeReportTest`. `verify-about-feature-gate` strips rust/go/node/python About via `about_lego_cli.py`. Feature specs require Tests + Fallback validation. Web↔Android i18n parity. Playwright RTL/reduced-motion/keyboard. WCAG token contrast (dark primary `#D3304E`). CSP on Vite preview + production meta. PWA share-target. UnifiedPush FOSS hook (no FCM). Settings JSON export. Lightroom `LrExportServiceProvider`. `/ideas` refuses silent `do all`; `/build` gate-locks the next feature.
- **Alternatives considered:** JSONObject on plain JVM (rejected: Android stub). CSP meta in source `index.html` (rejected: breaks Vite HMR). UnifiedPush connector Maven dep (deferred: hook + gradle ban until a distributor is in-tree).
- **Consequences:** Next `/build` row is M46-24 (debug recipe + `last-feature-gate.json` + 3-strike). Go About-without is skipped locally when `go` is missing (CI still runs it). HUMAN Scorecard/CII/DPIA remain after AGENT work.

### 2026-08-27 — /build M46 P0 + shared schemas
- **Status:** Accepted
- **Context:** `/build` on the M46 `/allideas` board. P0 was force-push honesty, Windows tool PATH, WSL1 bash, Sacred upgrade sim, plugin version.
- **Decision:** Deny `git push --force` even when session approved `git push`. Prepend `go`/`cargo` in `resolve-tools.sh`. Skip `System32` bash case-insensitively. Upgrade sim cherry-picks AREAS then asserts a child `AGENTS.md` marker. Plugin pack version follows `.template-version`. Shared Golden Path JSON schemas live under `schemas/golden-path/`.
- **Alternatives considered:** Allow force-push when `git push --force` is listed in session state (rejected for P0: `/push` never grants force). jsonschema PyPI dep (rejected: stdlib contract tests).
- **Consequences:** Next `/build` row is Node About + crash sanitize. HUMAN leftovers (Ollama, DPIA, CII, Scorecard) stay after AGENT work.

### 2026-08-27 — /allideas + M46 board
- **Status:** Accepted
- **Context:** `/ideas` caps at 5–8 (20 when asked). Sizeable automated progress needs an uncapped dump that can fill BUILD_PLAN, then `/build` on this template.
- **Decision:** Add `/allideas` (`allideas.md`, bare word `allideas`, `/AllIdeas` alias) with a `docs/help/ALLIDEAS.md` twin. Put the 2026-08-27 dump on the Template Maintainer board as M46. `/build --lane auto` prefers those 🔲 AGENT rows over weekly AUTO. Crash-proxy DPIA stays on M43; Scorecard badge and CII stay HUMAN.
- **Alternatives considered:** Raise `/ideas` cap (rejected: keeps a short ranked menu). Filename `all-ideas.md` (rejected: bare-word triggers are one word with no punctuation).
- **Consequences:** Next `/build` on this repo executes M46-1 (deny `git push --force`). Do not implement `/allideas` dumps until the user names numbers or says `board`.

### 2026-08-27 — /ideas round 2 (M45)
- **Status:** Accepted
- **Context:** `/ideas` asked for 20 in-scope items after M44. Health still showed closed RP #80 CI. Crash-proxy remains a DPIA item.
- **Decision:** Filter health CI off `release-please--branches--*`. Init installs commit-msg + sandbox copy. `--quick` runs action-ref format (API resolve stays full). Local Gradle patch apply via `UPDATE_GRADLE_PINS`. Rust/Go About+crash sanitize without extra crates. F-Droid and Lightroom in feature-gate. Plugin pack + Winget stub in CI. Radar writes gitignored AGENT suggestions. Encoding fail-closed is opt-in. Crash proxy stays disabled (`docs/CRASH_PROXY.md`).
- **Alternatives considered:** Live GitHub App proxy now (rejected: DPIA). `regex` crate on Rust Golden Path (rejected: keep zero-dep). Full GitHub API action resolve in `--quick` (rejected: needs network).
- **Consequences:** `--force` remains allowed when `git push` is session-approved (honesty test). HUMAN DPIA before crash-proxy enable.

### 2026-08-27 — /ideas ship hygiene (M44)
- **Status:** Accepted
- **Context:** Archive `docs(release)` opened 0.25.1; RP merge CI failed Unreleased order; worktree setup `npm ci`'d the primary tree; `.cursor-session-state.json` was untracked; RP PRs showed Dependency Review `ACTION_REQUIRED`.
- **Decision:** Drop `docs`/`chore` from `changelog-sections` so only feat/fix/perf/revert bump. `changelog_unreleased.move_unreleased_first` on `sync-template-version.sh`. Skip worktree installs unless `ROOT_WORKTREE_PATH` resolves to a different directory. Gitignore the JSON session file. Run `dependency-review-action` from `release-please.yml` and attach a check to the PR head. Print Android plugin pins beside the Gradle fallback. `/gates` always uses the canvas skill; `check-pre-commit-hooks.sh` fails locally, skips in CI.
- **Alternatives considered:** `hidden: true` for docs/chore (rejected: still bumps). `pull_request_target` for Dependency Review (rejected: untrusted checkout).
- **Consequences:** Closed leftover 0.25.1 (#80). Local `/gates` needs `pre-commit install --hook-type commit-msg`.

### 2026-08-27 — Ship v0.25.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after M42 local-first deps and M43 resource packing. `upd --apply` without `--max-bump minor` wrote GitHub Action majors; About-without restore `cp` hit Windows file lock after a passing gate.
- **Decision:** Cap apply at `--max-bump minor`; prefer `gh auth token` + `--token` for Release Please dry-run; child feature-gate skips missing optional toolchains; retry About restore then `git checkout` fallback. Empty Unreleased before push. Admin-merge Release Please #77 to **v0.25.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Apply GitHub Action majors (rejected: setup-java v6 / CodeQL fake tags). Fail `/prerelease` when Go is missing on the laptop (rejected: optional stack).
- **Consequences:** Template at 0.25.0. HUMAN leftovers: Ollama install, mcp.json copy, Dependabot frequency. ADB leftover: Android SDK licenses.

### 2026-08-27 — Local resource packing (M43)
- **Status:** Accepted
- **Context:** After M42, local deps were fast but `feature-gate` still ran stacks serially, worktrees reinstalled cold, `/best-of-n` was docs-only, and GPU was idle.
- **Decision:** RAM-capped parallel stack waves (CI max 2 slots; Android weight 2); worktree `npm ci --prefer-offline`; real `/best-of-n` and `/emulator` commands; Ollama on 127.0.0.1 with no cloud keys; emulator skip-if-no-SDK and never on `/ship`. GitHub CI remains post-push truth.
- **Alternatives considered:** Require Ollama or emulator for `/ship` (rejected). Auto-accept SDK licenses (rejected). `OLLAMA_ORIGINS=*` (rejected: LAN exposure). Content-hash skip-cache for bootstrap checks (rejected: stale green).
- **Consequences:** Agents use `check-local-compute`, `/best-of-n`, `/emulator`. Humans may install Ollama and accept Android licenses. `FEATURE_GATE_JOBS=1` is the low-RAM escape hatch.

### 2026-08-27 — Local-first dependency updater (M42)
- **Status:** Accepted
- **Context:** GitHub Dependabot PRs and `/prerelease` waiting on CI/Dependabot/Scorecard before push made releases slow.
- **Decision:** Primary path is local `upd-cli==0.6.2` (`scripts/update-deps.sh`, dry-run default) plus optional pinned depsonar MCP. `/ship` is `update-deps → prerelease → push → regress`. `pre-release-gate.sh --local` skips GitHub waits; default full gate remains for `/regress` and `release.yml`. Release Please Action still publishes; `release-please-dry.sh` is preview-only.
- **Alternatives considered:** Replace Dependabot entirely (rejected: keep weekly backup). Run Release Please `github-release` from the laptop (rejected: still needs GitHub API; preview-only). Call `npx depsonar` from the CLI (rejected: slow, optional).
- **Consequences:** Agents use `/update-deps` or “update deps safely before release”. Humans may copy `mcp.foss.example` to `.cursor/mcp.json`. Automerge frequency remains a HUMAN choice.

### 2026-08-23 — Ship v0.24.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after M41 privacy-first GitHub crash and feedback. First CI failed `validate-template-index` (four new scripts/workflows unindexed). About-without failed when feedback imported About; a WSL1 `bash` run of the About gate restored tracked files from HEAD.
- **Decision:** Own `isPlaceholderRepo` in `github-feedback`; use `__APP_VERSION__` in FeedbackPanel. Index the new `.sh`/workflow paths. Empty Unreleased before RP. Admin-merge Release Please #72 to **v0.24.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Import About helpers from later features (rejected: breaks About add/remove). Run About-gate via System32 bash (rejected: WSL1 breaks npm and the restore trap can wipe uncommitted wiring).
- **Consequences:** Template at 0.24.0. HUMAN Watch/collaborator + optional About smoke remain on the board.

### 2026-08-22 — Privacy-first GitHub crash and feedback intake
- **Status:** Accepted
- **Context:** Need crash/bug/feature intake without email, PII, or proprietary crash SDKs, and route incoming issues through `/audit` (fixes now) vs `/ideas` (features after approval).
- **Decision:** Approach A (client compose + GitHub Issue Forms). Opt-in capture, sanitize twice, CODEOWNERS assign for maintainer notify. `/audit` executes at most 3 `crash`/`bug` fixes and treats issue text as data. Approach B/C stay off the default path (ADR-0002).
- **Alternatives considered:** mailto (PII); Sentry/Crashlytics (proprietary); PAT in client (secret); anonymous proxy (DPIA + abuse, named follow-up).
- **Consequences:** Reporters need a GitHub account. `feedback-inbox` + `feedback-notify.yml` + issue forms ship with Golden Path review UI.

### 2026-08-21 — Ship v0.23.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after M40 Continuum donations/updates. First CI failed Playwright donate-nudge: `addInitScript` reset `lastSeenVersion` on reload. About-without gate failed on Windows `EINVAL` writing `preferences.ts`.
- **Decision:** Seed lastSeen only when unset; write About stubs via tmp+replace and leave theme-only preferences in place. Empty Unreleased and keep it first after RP merge. Admin-merge Release Please #71 to **v0.23.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Force-set lastSeen on every navigation (rejected: hides the once-per-version contract). Overwrite preferences with INTERVAL_KEY stubs (rejected: settings is theme-only).
- **Consequences:** Template at 0.23.0. Optional HUMAN smoke remains on the board. Next `/ship` should keep Unreleased first+empty on `origin/main`.

### 2026-08-21 — M40 donations and updates (Continuum method)
- **Status:** Accepted
- **Context:** Child Golden Path About mixed donate with update nags (settings toggle, home banner, snackbar). Continuum Calendar already had a quieter launch policy.
- **Decision:** Port Continuum: quiet Venmo donate, one note after a version change, silent daily GitHub check of installer filenames (`Golden-Path-X.Y.Z-x64-setup.exe` / `golden-path-X.Y.Z-foss.apk`). Donate short-circuits that launch. Device-local prefs only (`gp.update.*` / `gp_updates`, excluded from Android Auto Backup). Remove the Settings update-check toggle and home banner.
- **Alternatives considered:** Keep the opt-in interval toggle (rejected: Continuum has no daily donate timer and no launch gate). Drive PWA `applyPwaUpdate` from the GitHub installer prompt (rejected: About-only).
- **Consequences:** `docs/features/donations-updates.md` + `productUpdate` / `ProductUpdate` API. `OWNER/REPO` stays silent. HUMAN optional smoke remains on M40.

### 2026-08-20 — Ship v0.22.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after Android same-resolution high-refresh. First CI failed instrumented smoke: `preferredDisplayModeId` stayed 0 because `decorView.display` is null in `onCreate`.
- **Decision:** Apply the mode in `onStart` via `Activity.display` / `windowManager.defaultDisplay`. Empty Unreleased before push. Admin-merge Release Please #70 to **v0.22.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Keep the instrumented assertion optional (rejected: it caught a real no-op). Set the mode only in `onCreate` with `windowManager.defaultDisplay` (weaker on multi-display).
- **Consequences:** Template at 0.22.0. Child Android apps should vote display mode after the window attaches.

### 2026-08-18 — Ship v0.21.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after M38+M39. Pre-release green on `f54927e`; feat `8eab392` then `df322af` after `rp_merge_status` import failed once `PYTHONPATH` was stripped.
- **Decision:** Push feat + cwd-relative import fix; wait Ubuntu + Windows upgrade-sim; admin-merge Release Please #69 to **v0.21.0**. Fold leftover Unreleased onto the PR as comments. Archive M39 and empty Unreleased after merge (fold does not rewrite the RP branch). Codex skipped (no key/CLI).
- **Alternatives considered:** Leave leftover Unreleased under `[0.21.0]` (rejected: next `/ship` fails first+empty gates). `pull_request_target` for RP checks (rejected).
- **Consequences:** Template at 0.21.0. Next `/ship` should empty Unreleased in the prepare commit so RP does not carry bullets. SBOM attaches via `release` published workflow.

### 2026-08-17 — M39 /ideas Windows PATH + ship hygiene
- **Status:** Accepted
- **Context:** Fifth `/ideas` pass. Git Bash still missed `gh`; inherited `PYTHONPATH=scripts/lib` broke validate-bootstrap; leftover Unreleased blocked RP merge; Q&A REST create often SKIP'd.
- **Decision:** Shared `resolve-tools.sh` prepends Windows tool dirs and unsets `PYTHONPATH`. `agent-run` uses `child_env()`. Fold Unreleased onto the RP PR comment then empty. No `environment:` on CI/Security/CodeQL. GraphQL list + REST/GraphQL create for Q&A with a one-line HUMAN fallback. Archive M38; name Windows check on child Sprint 0 AUTO.
- **Alternatives considered:** Document PATH-only (rejected: every `gh` script still fails). `pull_request_target` for RP checks (rejected: untrusted checkout). Fail setup when Q&A API 422s (rejected: Settings fallback is enough).
- **Consequences:** Source `resolve-tools.sh` before `command -v gh`. Never export `PYTHONPATH=scripts/lib`. `/ship` comments leftover notes then empties Unreleased locally.

### 2026-08-17 — M38 /ideas ship-hardening
- **Status:** Accepted
- **Context:** Fourth `/ideas` pass after v0.20.0. Live `main` still required only five checks; Windows `jq` and leftover Unreleased still bit `/ship`.
- **Decision:** Fail `pre-release-gate` on missing Windows upgrade-sim; apply that check via `setup-github-repo`. Python-only `TEMPLATE_INDEX`. Skip RP wait on `ACTION_REQUIRED`. Split allowlisted `scripts/lib` to ≤150. Empty-Unreleased gate before RP merge. Archive Coach/M37/M36. Token on upgrade-sim jobs.
- **Alternatives considered:** Keep jq + CR strip (rejected: two paths). Leave lib allowlist (rejected: token-economy lie).
- **Consequences:** `/ship` now fails until protection matches the script. New `scripts/lib` files stay ≤150 with an empty allowlist.

### 2026-08-17 — Ship v0.20.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after three `/ideas` implement-all rounds. First pre-release gate was green on `01e21fc`; feat `14811be` then failed upgrade-sim (stamped purpose, pruned Android link, Windows `jq` CRLF).
- **Decision:** Keep portable-purpose assert on the template repo only; ignore doc links into pruned `modules/`/`examples/`; strip CR from `jq` paths in `validate-template-index`. Push fixes, wait for Ubuntu + Windows upgrade-sim, admin-merge Release Please #68 to **v0.20.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Skip full validate after prune (rejected: hides real child-repo doc breaks). Drop Windows upgrade-sim from required checks (rejected: that was the point of the ideas pass).
- **Consequences:** Template at 0.20.0; `/ship` must treat Windows `jq.exe` CRLF and post-prune links as first-class gates. SBOM attaches via `release` published workflow.

### 2026-08-17 — Implement /ideas backlog (required Windows check, coach twin)
- **Status:** Accepted
- **Context:** Third `/ideas` pass. Windows upgrade-sim existed but `/ship` and branch protection did not name it.
- **Decision:** Add the Windows job to required checks; ship `docs/help/COACH.md`; health notes dirty Unreleased; skip weekly AUTO rows after a green weekly-health run; Codespaces `verify.sh`; citation `date-released`; pin setup-python SHA; split `build_sprint` and gate new `scripts/lib` files at 150 lines (allowlist pre-existing oversize modules).
- **Alternatives considered:** Split every lib file in one pass (rejected: too much risk for `/build`). Fail file-limits on allowlisted modules (rejected: would block unrelated work).
- **Consequences:** `/ship` waits on Windows upgrade-sim once the job has run on HEAD. New `scripts/lib` modules must stay ≤150 lines.

### 2026-08-17 — Implement /ideas backlog (health, Windows CI, links)
- **Status:** Accepted
- **Context:** Second `/ideas` pass after the first eight items shipped locally. Health still pointed at child Sprint 0 on this template.
- **Decision:** Auto lane uses maintainer board when `bootstrap.config.json` still describes this template. Skip `pwsh` when missing; add `windows-latest` upgrade-sim. Split gate hints to JSON. Extend doc-link gate to root `*.md` + pre-commit. Best-effort Q&A category after Discussions enable.
- **Alternatives considered:** Keep auto=child on the template (rejected: wrong next step). Fail upgrade-sim without `pwsh` (rejected: bash path already proved). Require Q&A API success (rejected: Settings fallback).
- **Consequences:** `/coach` and `/ideas` on this repo name Ongoing Maintenance, not init-project. Child repos with their own purpose stay on the child playbook.

### 2026-08-17 — Implement /ideas backlog (8 items)
- **Status:** Accepted
- **Context:** `/ideas` ranked eight in-scope template items after v0.19.0. User asked to implement all.
- **Decision:** Ship Windows pyrepl env, CITATION.cff version sync, glossary, portable stamp copy, verify.sh hints, opt-in welcome issue, docs link gate, and Discussions enablement from `setup-github-repo`.
- **Alternatives considered:** Cursor-only purpose (rejected: portability). Welcome issue on by default (rejected: matches other post hooks). Fail setup if Discussions API cannot toggle (rejected: [HUMAN] Settings fallback).
- **Consequences:** `post_welcome_issue` stays false; `/ship` regress should finish on This Computer; glossary is REQUIRED.

### 2026-08-16 — Ship v0.19.0 (/ship)
- **Status:** Accepted
- **Context:** Coach layer (`2f77fb9`) plus portable first-run polish were unpushed; first pre-release CI wait failed because HEAD had no Actions run.
- **Decision:** Push feat commits, wait for CI/Security/CodeQL, re-run `pre-release-gate`, merge Release Please #67 to **v0.19.0**. Codex skipped (no key/CLI).
- **Alternatives considered:** Hold for Codex (rejected: skip is allowed). Invent per-tool rulebooks (rejected: AGENTS.md SoT).
- **Consequences:** Template at 0.19.0; SBOM attaches via `release` published workflow; batch commands are 24 atomic + 5 super.

### 2026-08-16 — Portable first-run (any agent IDE)
- **Status:** Accepted
- **Context:** First-time users needed a scripted tour and readable gate failures; the template was Cursor-weighted while Windsurf, Antigravity, and others already read `AGENTS.md`.
- **Decision:** Keep `AGENTS.md` Sacred. Generate thin pointers (`GEMINI.md` pointer-only, Windsurf, Cline, Aider, Continue). Ship `/tour` plus `docs/help/TOUR.md`. Add adapter drift gate, VS Code tasks, SUPPORT.md, CITATION.cff, good-first-issue, live badges, Codespaces link.
- **Alternatives considered:** Duplicate full rules into `.windsurfrules` / `.agents/agents.md` (rejected: drift and a second SoT). Register `/why` (rejected: `/coach` synonym only).
- **Consequences:** Edit `AGENTS.md` then `--sync-adapters`. Never put real rules in `GEMINI.md`.

### 2026-08-16 — Template Excellence / Coach Layer
- **Status:** Accepted
- **Context:** The template already had gates, memory, and Golden Paths; new users still got files without the industry *why*.
- **Decision:** Add `docs/BEST_PRACTICES.md` + `docs/FIRST_30_DAYS.md`, `/coach` + Welcome Tour, init what/why summary, optional FUNDING.yml/topics, and optional `justfile`s. Do not require `just` in CI. Do not register a second `/why` command.
- **Alternatives considered:** Fold the 30-day list into BEST_PRACTICES (rejected: token bloat); husky instead of just (rejected: pre-commit already covers hooks).
- **Consequences:** Batch-command count is 23 atomic + 5 super. `/bootstrap` ends with a tour. Child product READMEs gain For humans / For agents sections.

### 2026-08-16 — M37 gap close (verify, env, hooks)
- **Status:** Accepted
- **Context:** Checklist audit found core governance/CI present; remaining gaps were docker preflight, unimplemented post-hook flags, no root verify command, no env schema, no commit-msg enforcement, no Dockerfile, and no `.agent/memory` indexes.
- **Decision:** Extend the existing engine. `scripts/verify.sh` is the harness. Env validation is stack-agnostic JSON schema. Skills/memory under `.agent/` are indexes to `.cursor/skills/` and `DECISION_LOG.md` / `KNOWLEDGE_BASE.md`. Post install/test/git-init stay opt-in. Conventional Commits via pre-commit `commit-msg`, not Node-only commitlint.
- **Alternatives considered:** Duplicate skills into `.github/skills/` (rejected); husky + lint-staged (rejected: pre-commit already covers all stacks); auto-commit after init (rejected: destructive-ops).
- **Consequences:** `validate-bootstrap` requires env schema, Dockerfile, `.agent/` indexes, and `verify.sh`. Feature-gate fails if `.env.example` drifts from `env.schema.json`.

### 2026-08-16 — M36 bootstrap standards (AGENTS.md + lifecycle)
- **Status:** Accepted
- **Context:** Audit asked for a generator-style AGENTS.md engine, SDD stubs, security-by-default, manifest, and pre/post hooks. The repo already shipped SECURITY.md, CONTRIBUTING.md, CI, Dependabot, issue/PR templates, and `init-project`.
- **Decision:** Keep the GitHub Template + `init-project` model. Expand `AGENTS.md` as the canonical spec; generate thin Cursor/Claude/Copilot adapters; add `docs/spec.md` / `docs/plan.md`; add `bootstrap.config.json` plus preflight/post hooks and `PROJECT_CHECKLIST.md`. MIT remains default; Apache-2.0 is an init option for child repos. Do not auto-commit or auto-install deps.
- **Alternatives considered:** Separate yeoman-style generator CLI (rejected: would fork the template model); GitHub `- [ ]` checkboxes on the new checklist (rejected: repo-wide 🔲/✅/❌ convention).
- **Consequences:** `validate-bootstrap` requires SDD stubs, adapters, and engine unit tests. Child `AGENTS.md` stays Sacred on upgrade; adapters are Canon via `--sync-adapters`.

### 2026-08-16 — Ship v0.18.3 (/ship)
- **Status:** Accepted
- **Context:** Dependabot #64 Compose BOM bump on main; RP #66 already open
- **Decision:** `/ship` autofix + pre-release gate, then merge Release Please #66 to **v0.18.3**. Codex skipped (no key/CLI).
- **Alternatives considered:** Hold BOM for a later patch (rejected: CI including instrumented Android already green)
- **Consequences:** Template at 0.18.3; SBOM attaches via `release` published workflow

### 2026-08-16 — Ship v0.18.2 (/push)
- **Status:** Accepted
- **Context:** M35 HUMAN Scorecard/Dependabot/radar already on `main` @ `23254e8`; CI/Security/CodeQL green; RP #63 open
- **Decision:** Merge Release Please #63 to **v0.18.2** after local maintainer + pre-release gates (no extra prepare commit)
- **Alternatives considered:** Wait for RP auto-merge (blocked: no checks on release-please branch)
- **Consequences:** Template at 0.18.2; next quarterly radar 2026-11-15; Dependabot #64 left open

### 2026-08-15 — M35 Scorecard SARIF + Dependabot + radar
- **Status:** Accepted
- **Context:** Open HUMAN items after v0.18.1: Scorecard PinnedDependencies / TokenPermissions / VulnerabilitiesID; Dependabot #58–#61; quarterly radar (last report 2026-06-30)
- **Decision:** Job-scope write tokens (`permissions: read-all` at workflow level). Keep `@vX.Y.Z` for GitHub-owned actions. Treat VulnerabilitiesID as stale (hono/nanoid/postcss already patched in 0.18.0). Merge green Dependabot PRs after rebase; rebase #61 (stale web lockfile). Radar max new score is 6 — no BUILD_PLAN row.
- **Alternatives considered:** SHA-pin every `actions/*` (rejected: conflicts with `validate-workflow-actions` + existing policy); add Design Mode / Canvas now (rejected: score 6, below ≥9 suggest threshold)
- **Consequences:** TokenPermissions should clear on next Scorecard run; PinnedDependencies remain accepted; next quarterly radar due 2026-11-15

### 2026-08-15 — Ship v0.18.1 (/push)
- **Status:** Accepted
- **Context:** M35 Windows Store `python3` hang + About-gate restore ready; first `PY="py -3"` broke `"$PY"` in Dependabot count
- **Decision:** Resolve `PY` to `sys.executable` from `py -3`; merge Release Please #62 to **v0.18.1** after CI green on `b4fca9c`
- **Alternatives considered:** Leave `PY="py -3"` and unquote all callers (rejected: `"$PY"` is the safe pattern); wait for RP auto-merge (blocked: no checks on release-please branch)
- **Consequences:** Template at 0.18.1; Scorecard SARIF and Dependabot PRs #58–#61 stay HUMAN

### 2026-08-15 — Audit M35 Windows Python resolver
- **Status:** Accepted
- **Context:** `/ship` autofix hung because `command -v python3` resolved to the Microsoft Store stub under `WindowsApps`
- **Decision:** Add `scripts/lib/resolve-python.sh` (skip Store stub; prefer `py -3`) and source it from gate/autofix scripts; restore About slice from `git checkout HEAD` if the verify-about backup is missing
- **Alternatives considered:** Document-only workaround (rejected: every Windows gate still hangs); require a `python3` symlink in PATH (rejected: Store alias still wins)
- **Consequences:** Local gates on This Computer no longer stall on the stub; HUMAN still owns Scorecard SARIF and Dependabot PRs #58–#61

### 2026-08-15 — Ship v0.18.0 (/ship)
- **Status:** Accepted
- **Context:** M34 prior-art steals ready; pre-release gate blocked on High `extract-zip` (no upstream patch) via LHCI → puppeteer-core
- **Decision:** Override `@puppeteer/browsers` >=3.2.0 (uses `modern-tar`); lock optional peer `proxy-agent` >=8.0.2 so CI `npm ci` matches; bump `hono`/`postcss`/`nanoid`; merge Release Please #56 to **v0.18.0**
- **Alternatives considered:** Dismiss extract-zip as dev-only (rejected: gate requires zero High); vendor a patched fork (rejected: no patch exists)
- **Consequences:** Template at 0.18.0; honesty labels + scratchpad/handoff ship; Codex skipped (no key/CLI)

### 2026-08-14 — Prior-art thin steals (M34)
- **Status:** Accepted
- **Context:** Compared CopperDogma, Barony, Sciensoft, and wshobson/agents against this Cursor-first template. Need mechanisms without vendoring those trees or an 80k playbook.
- **Decision:** Ship honesty labels, parallel handoff stub, Canon/Mixed/Sacred upgrade column, OWASP LLM walk, scratchpad reset, optional marketplace pointer, and bootstrap-doctor alias. Number as **M34** (plan draft said M30; that sprint is already archived).
- **Alternatives considered:** Vendor Barony/`baron` (rejected: second product + PyPI dep); install wshobson catalog by default (rejected: token bloat); Sciensoft one-file playbook (rejected: 300/150 caps).
- **Consequences:** Hooks stay fail-open and labeled; child `AGENTS.md` / init prompt stay Sacred; no new CI scanner or marketplace install on the FOSS default path.

### 2026-08-12 — Ship v0.17.0 branding kit (/ship)
- **Status:** Accepted
- **Context:** Child repos need replaceable logos/colors and pitch-quality READMEs without overwriting the template README
- **Decision:** Ship `branding/` pack + mode-gated `generate-project-readme.py` (`template` preview only; `product` writes root README); extend token sync for official-colors and asset distribution; merge Release Please #55 to **v0.17.0**
- **Alternatives considered:** Generate logos from tokens only (rejected: humans replace art files); always overwrite root README (rejected: clobbers template guide)
- **Consequences:** Sprint 0 fills `product.json` then generate; upstream keeps `mode: template`; store PNGs remain human/ADB exports

_Seed template ADR: `docs/adr/0000-template-baseline.md`. Child repos use `docs/adr/0001-core-architecture.md`._

### 2026-08-10 — Ship v0.16.0 (/ship)
- **Status:** Accepted
- **Context:** Need third-party review + broader autofix before release; `/ship` should stay one command
- **Decision:** Codex read-only reviewer (opt-in CI + `/codex-review`) feeds `CODE_REVIEW.md` → Cursor `/fix`; expand `/prerelease` with multi-stack autofix; merge Release Please #51 to **v0.16.0**
- **Alternatives considered:** Codex writes patches in CI (rejected: destructive-ops / FOSS spend control); chain Codex into every `/maintain` (rejected: API cost)
- **Consequences:** `/ship` runs autofix + optional Codex + hard gate; enable Codex CI by copying workflow example + `OPENAI_API_KEY`

### 2026-08-01 — Ship v0.15.2 (/ship)
- **Status:** Accepted
- **Context:** Plan Mode left risks as open questions; Dependabot High blocked pre-release (js-yaml, then postcss)
- **Decision:** Require Issue→Resolution Critique in always-applied rules + `/plan`; override patched npm transitive CVEs; merge Release Please #50 to **v0.15.2**
- **Alternatives considered:** Soft "list risks" Critique (rejected: humans still had to chase resolutions); defer brace-expansion/postcss (rejected: pre-release gate requires zero Critical/High)
- **Consequences:** Agents must bake mitigations into plan todos; template at 0.15.2 with SBOM release assets

### 2026-07-22 — Ship v0.15.0 (/ship)
- **Status:** Accepted
- **Context:** `/ship` after M33 + local-first compute; first CI failed on duplicate `## [Unreleased]`; web tests failed on Node 25+ localStorage stub
- **Decision:** Polyfill Storage in vitest setup (KB-011); collapse stale Unreleased; merge Release Please #37 to **v0.15.0**
- **Alternatives considered:** `--no-webstorage` only (rejected: may break older Node CI); leave duplicate Unreleased (rejected: gate hard-fail)
- **Consequences:** Template at 0.15.0 with Cursor worktrees/permissions/skills/plugin pack and local-first parallelism

### 2026-07-21 — Local-first compute on This Computer
- **Status:** Accepted
- **Context:** Agents defaulted toward serial work or Cloud handoff even when the desktop has many cores
- **Decision:** Ship `local-compute.mdc` + sessionStart CPU reminder; parallelize independent `validate-bootstrap` checks via `run_checks_parallel.py` (`BOOTSTRAP_CHECK_JOBS`); pytest-xdist `-n auto`; Gradle `--parallel`; document `/scope` + worktrees/`/best-of-n` as the local default over Cloud Agents
- **Alternatives considered:** Always Cloud Agents for parallelism (rejected: wastes local hardware and costs credits); unbounded bash `&` in validate-bootstrap (rejected: harder error aggregation on Windows)
- **Consequences:** Quick bootstrap checks use all cores (e.g. jobs=CPU count); agents are steered to concurrent Task/worktrees when local

### 2026-07-21 — Cursor 3.9–3.11 FOSS integration (M33)
- **Status:** Accepted
- **Context:** Cursor added native worktrees setup, Auto-review `permissions.json`, Skills direction, CLI/GHA, side chats, Design Mode, cloud conversation hooks, Automations, and plugin packaging; registry lagged at 2026-06-30
- **Decision:** Ship FOSS live `worktrees.json` + fail-soft OS setup, committed `permissions.json` (dual layer with hooks), four new skills + checker atomic update, CLI workflow under `.github/workflow-examples/` (never auto-run), plugin via pack-to-`dist/cursor-plugin` (no repo-root symlink); keep commercial as examples (cloud hooks, Automations recipes, Bugbot Autofix map)
- **Alternatives considered:** Custom plugin paths into `.cursor/` (rejected: discovery risk); whole-repo plugin symlink (rejected: double-load); `.example.yml` under `workflows/` (rejected: GHA may load it); weaken shell hook for Auto-review (rejected: hooks stay hard FOSS enforcement)
- **Consequences:** `check-cursor-integrations` requires seven skills + worktrees/permissions; `/best-of-n` documented beside parallel-lock worktrees; Cloud Agents still ignore Run Modes

### 2026-07-12 — Pre-release gate Dependabot counter + FOSS MCP check
- **Status:** Accepted
- **Context:** `/push` pre-release `--strict` failed: Dependabot alerts API used unsupported `page=` form; FOSS integrations check failed whenever gitignored `.cursor/mcp.json` existed locally
- **Decision:** Count alerts via `gh api --paginate` query string; treat live `mcp.json` as OK unless `git ls-files` shows it tracked; multi-stack `--strict` skips missing optional toolchains
- **Alternatives considered:** Require `security_events` refresh always (rejected: false failures blocked release); ban local MCP (rejected: contradicts CURSOR_INTEGRATIONS activation)
- **Consequences:** Maintainer gates pass with local MCP enabled; Release Please #36 published v0.14.1

### 2026-07-12 — Dependabot automerge CI gap (M32)
- **Status:** Accepted
- **Context:** Merges via `GITHUB_TOKEN` (`app/github-actions`) do not start `push` workflows; `main` tip after Dependabot merges had zero CI runs; weekly health failed waiting for missing runs
- **Decision:** Prefer optional `AUTOMERGE_TOKEN` PAT for Dependabot/Release Please merge; add `workflow_dispatch` to CodeQL + Security Scan; `check-github-ci.sh --dispatch-if-missing` (weekly health uses it with `actions: write`); prefer Git Bash in `agent-run.py` on Windows
- **Alternatives considered:** Require PAT only (rejected: blocks FOSS template without secrets); SHA-pin all actions for Scorecard (deferred: conflicts with documented `@vX.Y.Z` policy)
- **Consequences:** Weekly health can self-heal missing runs; post-merge CI still needs HUMAN required-status-checks + optional PAT for true push triggers

### 2026-07-02 — Quiet agent shell (hooks Python + agent-run)
- **Status:** Accepted
- **Context:** Cursor Agent shell execution opened `.sh` hook and script tabs, stealing editor focus while users typed
- **Decision:** Migrate hooks to Python; add `scripts/agent-run.py` for agent gate invocations; ship `.vscode/settings.json` anti-reveal defaults; document KB-010
- **Alternatives considered:** Disable hooks globally (rejected: loses destructive-op guard); rewrite all scripts to PowerShell (rejected: scope); `pythonw.exe` for hooks (rejected: breaks stdout JSON)
- **Consequences:** Agent-facing commands no longer contain `.sh` paths; underlying bash scripts unchanged for CI/humans

### 2026-07-01 — Cursor hook smoke isolation (M31)
- **Status:** Accepted
- **Context:** M31 audit found `check-cursor-hooks.sh --smoke` false-pass when `.cursor-session-state.json` already listed `git push` in `destructive_ops_approved`
- **Decision:** Smoke test clears session approvals before deny assertion; validate hook scripts require shebang on line 1
- **Alternatives considered:** Ignore local session state in smoke (rejected: hides real deny-path bugs); require empty session file (rejected: breaks dev workflow)
- **Consequences:** `--smoke` is deterministic in CI and locally; invalid hook scripts fail validate-bootstrap early

### 2026-06-30 — Cursor hooks as enforcement layer (M30)
- **Status:** Accepted
- **Context:** M27 rejected `beforeSubmitPrompt` hooks; rules alone cannot block destructive shell commands at runtime
- **Decision:** Ship FOSS-safe project hooks (`beforeShellExecution`, `afterFileEdit`, `subagentStart`, `sessionStart`, `beforeMCPExecution`); fail-open guards; session `destructive_ops_approved` for `/push`/`/ship`; opt-out via `<!-- cursor-hooks: off -->`
- **Alternatives considered:** Prompt-rewrite hooks (rejected per M27); broad shell blocklists (rejected: blocks legitimate agent work)
- **Consequences:** `check-cursor-hooks.sh --smoke` in validate-bootstrap; complements `destructive-ops.mdc` without token bloat

### 2026-06-20 — Repo-wide checklist status markers
- **Status:** Accepted
- **Context:** BUILD_PLAN and scattered checklists used mixed ⬜ / `- [ ]` / ✅ formats; inconsistent in Markdown Preview vs source
- **Decision:** Standardize on 🔲 open · ✅ done · ❌ blocked emoji markers repo-wide; document in `BUILD_PLAN.md` legend and agent read order
- **Alternatives considered:** GitHub `- [ ]` task lists (rejected: poor Preview readability and agent parsing); keep ⬜ white square (rejected: visually similar to ✅ in some fonts)
- **Consequences:** All new checklist rows use emoji; `agent-progress.sh` accepts legacy ⬜ for child repos during transition

### 2026-06-18 — Release automation hardening (M29)
- **Status:** Accepted
- **Context:** v0.11.0 release lacked SBOM assets (GITHUB_TOKEN cannot chain `release` → `release.yml`); Release Please skipped `extra-files`; `health-check.yml` registered as path name caused 0-job push failures
- **Decision:** `release-please.yml` runs `sync-template-version.sh` on release PR branches and dispatches `release.yml` on `release_created`; rename workflow to `weekly-health-check.yml`; fix sync script for Windows Git Bash
- **Alternatives considered:** PAT with workflow scope for release chaining (rejected: secrets management); manual SBOM backfill only (rejected: repeated human step each release)
- **Consequences:** Release Please needs `actions: write`; future releases should ship SBOM assets without manual dispatch

### 2026-06-17 — Batch instruction templates (M27)
- **Status:** Accepted
- **Context:** Agents and child-repo owners needed repeatable shortcuts for bootstrap, verify, build, ship, and maintenance workflows without re-pasting long prompts
- **Decision:** Ship 25 slash commands in `.cursor/commands/` (20 atomic + 5 super), bare-word expansion via `batch-commands.mdc`, human cheat sheet at `docs/help/BATCH_COMMANDS.md`, registry at `docs/BATCH_COMMANDS.md`; `/push` and `/ship` grant explicit push approval
- **Alternatives considered:** `beforeSubmitPrompt` hook for bare words (rejected: Cursor API cannot rewrite prompts); single mega-doc for humans and agents (rejected: overwhelms first-time users)
- **Consequences:** `alwaysApply` rule adds ~25 lines per session; `check-batch-commands.sh` prevents registry drift; child repos cherry-pick via `UPGRADING_FROM_TEMPLATE.md`

### 2026-06-30 — Autonomous /build with grouped human section
- **Status:** Accepted
- **Context:** `/build` halted on HUMAN/ADB rows; humans needed a single review block after automation; child repos need scripted attempts before manual follow-up
- **Decision:** Add `build-sprint-status.sh`, `attempt-build-plan-row.sh`, and `HUMAN_BACKLOG.md` (failure-only); restructure BUILD_PLAN with `#### Human & device (after automation)`; AGENT/AUTO runs first, then automation attempts on grouped human rows
- **Alternatives considered:** Skip human rows entirely during /build (rejected: loses automation catalog value); keep human rows interleaved in Sequential (rejected: hard to review after automation)
- **Consequences:** Child repos must place HUMAN/ADB rows in the grouped section; `<!-- no-auto-approve -->` disables autonomous ADR ack

### 2026-06-13 — @lhci/cli npm overrides for transitive CVEs
- **Status:** Accepted
- **Context:** Lighthouse CI (`@lhci/cli`) bundles transitive dependencies (`tmp`, `uuid`) with known CVEs; no patched `@lhci/cli` release available at triage time
- **Decision:** Add npm `overrides` in `examples/web/package.json` forcing `tmp >= 0.2.6` and `uuid >= 11.1.1`; document in KB-007
- **Alternatives considered:** Dismiss Dependabot alert (rejected: hides real risk); remove Lighthouse CI job (rejected: loses performance gate)
- **Consequences:** Lockfile must be regenerated after override changes; overrides should be removed when `@lhci/cli` ships fixed dependencies

### 2026-06-13 — Ship all optional ecosystem modules (M3)
- **Status:** Accepted
- **Context:** Sprint M3 asked whether to ship Lightroom, Rust, and Go optional modules in the template maintainer repo
- **Decision:** Ship all three with Golden Path stubs, MODULE.md guides, and path-gated CI jobs (`lightroom`, `rust`, `go`) that skip when child repos remove the directories
- **Alternatives considered:** Lightroom-only (rejected: Rust/Go stubs are low-cost and popular); defer all optional modules (rejected: COMPLETED_TASKS M3 work already landed)
- **Consequences:** Template CI runs more jobs on `main`; child repos can delete unused `examples/` folders to skip jobs via `hashFiles` guards

### 2026-09-10 — M61 board from allideas 161–200
- **Status:** Accepted
- **Context:** After M58–M60 archive, `/allideas` dump 161–200 filled maintainer milestone M61 (back/nav, gates, template inherit).
- **Decision:** Execute M61 AGENT rows via `/build`; keep Open PRs sync + Release Please merge as HUMAN; archive M61 after `smoke-sprint --require`. Pointer: allideas dump → this log (M58–M61).
- **Alternatives considered:** Fold 161–200 into M60 (rejected: smoke already passed). Skip device HUMAN/ADB (rejected: backlog instead).
- **Consequences:** Prefer archive M58 before opening M62 (#199). Do not push from `/build` without `/push`.
## /push prepare 1.3.0 (2026-09-11)

- What: Commit outstanding M58–M61 + waiting-row automation + UnifiedPush E2E; empty CHANGELOG Unreleased; push main; merge RP #106.
- Validated: `pre-release-gate.sh --local`, `verify-about-feature-gate.sh`, license compliance, repo hygiene, README health.
- Deferred: Lightroom Plug-in Manager load (needs Adobe); RP merge waits on required checks after push.
## Autonomous /build approval (2026-09-11T13:28:49+00:00)

- Mint 22 Cinnamon smoke + ADR auto-approval (inventory + dry-run)
## Autonomous /build approval (2026-09-11T15:22:17+00:00)

- PWM take-over declined; active daemons: coolercontrold
## Autonomous /build approval (2026-09-11T15:22:17+00:00)

- Started OpenRGB --server on 127.0.0.1
## Autonomous /build approval (2026-09-12T22:45:54+00:00)

- pkexec install-support.sh --apply succeeded
## Visible installed GUI (2026-09-13)

- What: Blank `chromaflow-gui` was (1) `cargo --release` without `tauri/custom-protocol` so WebKit opened `127.0.0.1:1420`, (2) Vite `/assets/` absolute paths, (3) NVIDIA DMA-BUF. Fix: `--features custom-protocol`, `base: "./"`, `.deb` `npm run build`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
- Validated: `tests.test_chromaflow_tauri` / packaging; rebuild `.deb`; live window paint.
- Deferred: compositing-mode off only if DMA-BUF-only still blanks.
## ITE pwm4 + NVIDIA DISPLAY (2026-09-13)

- What: Auto-calibrate showed 0 RPM on FAN4/FAN5 while it87952 `fan4` ran ~3100 RPM with no card (`pwm4` ENODATA until enable=1). `chromaflowd` applied 8 ITE headers and `gpu 0` because `nvidia-settings` had no DISPLAY.
- Decision: List unread `pwm*` when the sysfs node exists; pair `fanN`↔`pwmN`; name pwm4 `FAN7_PUMP`; set `DISPLAY=:0` / `XAUTHORITY` on the user unit and in the nvidia-settings helper. Empty SYS_FAN2/3 / CPU_OPT stay 0 RPM (no device). The ~3100 RPM header is a tach; motherboard PWM did not map 1:1 to RPM.
- Validated: rust hwmon unread-pwm test; cooling UI FAN7/fanN pairing; daemon unit DISPLAY; live takeover after `.deb`.
- Deferred: 3-pin DC mode (no `pwm_mode` sysfs); USB AIO CLI (liquidctl only lists Fusion RGB).
## Ship lighting engine (2026-09-13)

- What: Engine was a download button, not in the `.deb`. MIT still cannot vendor OpenRGB C++. Package hashed AppImage at `/usr/libexec/chromaflow/OpenRGB.AppImage`; `chromaflow daemon --sdk` + `APPIMAGE_EXTRACT_AND_RUN=1` (FUSE mount is blocked under the user unit).
- Validated: `chromaflow devices` listed 3 SDK controllers; unit active; 127.0.0.1:6742 listen.
- Deferred: in-process OpenRGB still needs a relicense ADR.
