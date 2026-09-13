# Hardware detection (Linux vs Windows OpenRGB)

ChromaFlow does not claim to match Windows OpenRGB device counts by “scanning harder.” Typical gaps are **permissions, missing kernel modules, or no Linux control channel**.

## Why OpenRGB often sees more on Windows

| Gap id | What happened | What ChromaFlow does |
|--------|---------------|----------------------|
| `openrgb_sandboxed` | OpenRGB is Flatpak/bwrap so USB HID is hidden even when the SDK is up | Recommend native `openrgb --server --server-host 127.0.0.1` (distro package or unsandboxed AppImage); do not fake USB devices |
| `usb_rgb_not_in_openrgb` | USB HID RGB/ARGB exists that has no Linux backend yet | Skip Fusion `048d:5702` when Aorus/liquidctl already drives it, Arena/Prime VID backends, and Keychron when the SDK lists it; leftover HID is research-only (no guessed reports) |
| `udev_hidraw` | USB HID exists but `/dev/hidraw*` is root-only | Write per-VID `60-chromaflow.rules` (`MODE=0660` `GROUP=plugdev` `uaccess`); not a missing `.ko` |
| `i2c_dev_missing` | `i2c-dev` or SMBus adapter not loaded or not permitted | Recommend `i2c-dev` + `i2c-i801` / `i2c-piix4` from the allowlist. Gigabyte X570/X570S: ACPI `\GSA1.SMBI` can block PIIX4 at `0xb00` until `acpi_enforce_resources=lax`; DIMM JC-42 temps need that SMBus |
| `missing_module` | Board needs `nct6775`, `it87`, `nct6775-i2c`, `i2c-nvidia-gpu`, etc. | Checklist from YAML; experimental behind Advanced. There is no `i2c-nct6775` module. If `i2c-nvidia-gpu` is loaded, both RGB I2C extras count as present. |
| `ite_primary_missing` | Gigabyte dual-ITE: in-tree `it87` bound **IT87952E** only | Support Extra kernel support lists `it87-dkms` first. Present is DKMS srcversion, not in-tree `it87`. Do not `modprobe it8689`. ChromaFlow does not ship `.ko` or add CoolerControl apt |
| `nvidia_fans_query` | `nvidia-settings` listed no fans | Needs a `DISPLAY`; `chromaflowd` sets `DISPLAY=:0` and `XAUTHORITY`. Hybrid 4090 fans often read 0% idle. Cooling still shows fans when the query succeeds. `nvidia-smi` fan.% is the same card fan, not extra headers |
| `windows_only_protocol` | Device only has a Windows dump / no Linux backend | UI: “No Linux backend yet” + link to OpenRGB issues — **do not fake control** |
| `no_os_control_channel` | 12 V hub or dumb splitter with no PWM/ARGB controller the OS can talk to | Explain that Install + Rescan cannot create a protocol |
## CoolerControl backends (read-only scan)

CoolerControl discovers **hwmon**, **liquidctl**, **NVML**, and GPU CLI tools. It does not ship `.ko`. ChromaFlow maps those without vendoring CoolerControl or adding their apt:

| CoolerControl path | ChromaFlow |
|--------------------|------------|
| `it87-dkms` (frankcrawford) | Support extra `it87-dkms` first |
| in-tree `nct6775` / `it87` / `w83627hf` | Extras + YAML; Winbond `w83627hf` stays YAML-only (this X570S is ITE) |
| `nvidia-settings` / `nvidia-smi` / NVML | GPU fans via `nvidia-settings`; temps via `nvidia-smi`. No NVML (closed SDK) |
| `amdgpu` hwmon PWM | YAML `amdgpu`; this host is NVIDIA |
| `liquidctl` USB AIO | Extra kernel support `liquidctl` (apt `liquidctl`). Present = package or `/usr/bin/liquidctl`. CPU AIO stays on its own controller until that CLI lists speeds |
| `thinkpad_acpi` / `asus-wmi` / `dell-smm-hwmon` | YAML, DMI-gated |
| Root `sensors-detect` via `/dev/port` | Not used. GUI never root |
## GPU radiator cases

1. **GPU header** — NVIDIA: `nvidia-settings` fan:0 / fan:1 (this 4090 hybrid). AMD `amdgpu` hwmon `pwm*` when present. We never force proprietary vs nouveau. Writes use `GPUTargetFanSpeed` after confirm; never silent 0%; failsafe `GPUFanControlState=0`. Coolbits may be required to take over.
2. **AIO / pump hub** — USB HID or USB+i2c. Often `liquidctl` or OpenRGB, after udev.
3. **ARGB hub on a SATA/molex cable** — lighting may be OpenRGB HID; fan tach/PWM may be absent (`no_os_control_channel`).
4. **Chassis hub with no MCU** — neither Linux nor Windows software can PWM the fans.

## Uncontrolled device reports

Lighting **Uncontrolled devices** → **Report** opens a GitHub issue labelled `device`. Inbox: [open device issues](https://github.com/edwardlthompson/chromaflow/issues?q=is%3Aissue+label%3Adevice+is%3Aopen). Process per [`docs/features/device-reports.md`](features/device-reports.md).

## Probe inputs (selection only)

`lspci`, `lsusb`, and DMI board name **select rows already in YAML**. They never become `modprobe` arguments.

## Conflicts

If `fancontrol`, `coolercontrold`, or `fan2go` is enabled or running, inventory sets `conflicts[]`. This milestone only warns. A future daemon must not take PWM without an explicit confirm.
