# ADR-0017: Host hardware gauges without OpenRGB plugins

- **Status:** Accepted
- **Date:** 2026-09-12
- **Deciders:** ChromaFlow lighting follow-up
- **Related:** ADR-0016 (host Direct effects), ADR-0007 (MIT, no in-process GPL)

## Context

OpenRGB’s Hardware Sync plugin maps sensors to LED colors. It is a separate GPL program. AGENTS.md forbids Windows OpenRGB plugins. Users want per-component green→yellow→red meters (CPU, GPU, combined, RAM, disk) and device routing (motherboard=CPU, GPU AIO=GPU, keyboard+speakers=combined). This host has no nvidia/amdgpu hwmon. Inventory’s full OpenRGB DATA dump also rebuilt the Lighting tree and hitched Cinnamon.

## Decision

ChromaFlow samples hwmon temps, `/proc/meminfo`, and `df -P /` in-process. When hwmon has no GPU group, `hardware_gauges` may run **cached** `nvidia-smi --query-gpu=temperature.gpu,utilization.gpu` (2 s TTL). That query is never on the LED frame path and never in `collect_inventory`. CPU load is a `/proc/stat` delta. Host Direct effects:

- `Hardware gauges` — per device: Aorus/Fusion=CPU, 4090/AIO=GPU, Keychron/Arena=combined
- `CPU` / `GPU` / `Combined` / `RAM` / `Disk` — LED color follows the cube Temp/Usage toggle (not separate temp/usage effects)

Hue 120→0. Combined uses the **average** of CPU and GPU ratios (35 °C = 0, 90 °C = 1; usage 0–100%). The Lighting cube Temp/Usage toggle is the only switch for that ratio. RAM °C is jc42 (DDR4) or spd5118 (DDR5) on the chipset SMBus, or a channel whose hwmon label is DIMM — not unlabeled ITE thermistors. On Gigabyte X570S AORUS MASTER the AMD SMBus (PIIX4 at `0xb00`) is reserved by ACPI `\GSA1.SMBI`, so `i2c-piix4` does not bind until the kernel boots with `acpi_enforce_resources=lax`; then `jc42` can attach. No plugin, no NVML, no PWM. OpenRGB probe and `liquidctl list` stay cached 15 s; inventory poll is 15 s and **skipped while Lighting is open** so WebKit does not rebuild 108-key boards. Identical host frames are not stamped onto the layout. The hardware cube matches the color-picker square (bars only).

## Alternatives considered

- Load OpenRGB Hardware Sync — rejected (GPL plugin, Windows path)
- Spatial bar on the keyboard — rejected; the 1-LED AIO cannot show a bar
- Uncached `nvidia-smi` per frame — rejected; hitch class. Cached 2 s on the meter path only is accepted because this machine has no GPU hwmon.

### Critique

| Issue | Resolution |
|-------|------------|
| Null/empty sensors | `Option` fields; last `ui.gauges` then green; tests empty meminfo / parse N/A smi |
| Network timeout | Gauges are local sysfs/`df`/`nvidia-smi`; probe cache + try_lock for SDK |
| Race vs lighting_sync | try_lock never waits; 10 Hz motion / skip unchanged gauge frames |
| Unhandled exceptions | Failed `hardware_gauges` keeps last ratios; smi spawn failure is `None` |
| PWM | `gauges.rs` / `nvidia_smi.rs` have no pwm paths |
| ACL | `hardware_gauges` on existing `allow-inventory` |
| Inventory hitch | 15 s poll; skip on Lighting; skip identical `lastPaint` |

## Consequences

Host effects stay the lighting catalog. Probe list may be 15 s stale after unplug. GPU °C can lag 2 s. Disk used% can lag 30 s. PWM unchanged.
