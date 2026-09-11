# Hardware detection (Linux vs Windows OpenRGB)

ChromaFlow does not claim to match Windows OpenRGB device counts by “scanning harder.” Typical gaps are **permissions, missing kernel modules, or no Linux control channel**.

## Why OpenRGB often sees more on Windows

| Gap id | What happened | What ChromaFlow does |
|--------|---------------|----------------------|
| `udev_hidraw` | USB HID exists but `/dev/hidraw*` is root-only | Recommend OpenRGB-style udev + `uaccess` / `plugdev` |
| `i2c_dev_missing` | `i2c-dev` or SMBus adapter not loaded or not permitted | Recommend `i2c-dev` + `i2c-i801` / `i2c-piix4` from the allowlist |
| `missing_module` | Board needs `nct6775`, `it87`, `i2c-nct6775`, `i2c-nvidia-gpu`, etc. | Checklist from YAML; experimental behind Advanced |
| `windows_only_protocol` | Device only has a Windows dump / no Linux backend | UI: “No Linux backend yet” + link to OpenRGB issues — **do not fake control** |
| `no_os_control_channel` | 12 V hub or dumb splitter with no PWM/ARGB controller the OS can talk to | Explain that Install + Rescan cannot create a protocol |

## GPU radiator cases

1. **GPU header** — fan/pump on the card’s header. Control only if drm/nvml **hwmon** exposes `pwm*`. AMD `amdgpu` often does; NVIDIA may need the kernel module the user already uses. We never force proprietary vs nouveau.
2. **AIO / pump hub** — USB HID or USB+i2c. Often `liquidctl` or OpenRGB, after udev.
3. **ARGB hub on a SATA/molex cable** — lighting may be OpenRGB HID; fan tach/PWM may be absent (`no_os_control_channel`).
4. **Chassis hub with no MCU** — neither Linux nor Windows software can PWM the fans.

## Probe inputs (selection only)

`lspci`, `lsusb`, and DMI board name **select rows already in YAML**. They never become `modprobe` arguments.

## Conflicts

If `fancontrol`, `coolercontrold`, or `fan2go` is enabled or running, inventory sets `conflicts[]`. This milestone only warns. A future daemon must not take PWM without an explicit confirm.
