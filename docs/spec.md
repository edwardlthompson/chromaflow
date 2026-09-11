# Product Specification

> Status markers: 🔲 open · ✅ done · ❌ blocked.

## Overview

**Product:** ChromaFlow  
**Purpose:** Single Linux Mint desktop app for fan/pump curves and RGB/LED control, with detection as close as Linux allows to Windows OpenRGB, plus Install detection support + rescan.  
**Users:** Cinnamon desktop users on Mint 21/22 who want cooling and lighting without running the GUI as root.

## Functional Requirements

| ID | Story | Acceptance |
|----|-------|------------|
| FR-1 | As a user I run `chromaflow sensors` / `devices` / `rescan` so I see JSON inventory | hwmon + hidraw + i2c + liquidctl-if-present + OpenRGB localhost probe; no PWM writes |
| FR-2 | As a user I run Support dry-run so I see a checklist | JSON would_* fields; no apt/modprobe/udev mutation |
| FR-3 | As a user I open the GUI tabs Cooling / Lighting / Profiles / Support | English UI; curve placeholder is display-only; Support shows dry-run |
| FR-4 | As a user I am told why a device is missing | `gaps[]` uses HARDWARE.md ids; Windows-only is not faked |
| FR-5 | As a user I am warned about other fan daemons | `conflicts[]` includes fancontrol, coolercontrold, fan2go |

## Non-Functional Constraints

- MIT + NOTICE; no Fan Control or OpenRGB source copies
- No bundled kernel modules
- GUI/CLI refuse euid 0
- Allowlist regex for module and apt names
- File budgets: 300 lines static data, 150 lines pure logic
- Core inventory works offline
- Tests use fixture sysfs; CI does not modprobe

## Architecture & Data Flow

See [`docs/ARCHITECTURE.md`](ARCHITECTURE.md). Hexagonal: domain inventory types; adapters for sysfs, YAML, TCP probe, process PATH.

## Test-first rule

Every feature in `docs/plan.md` / BUILD_PLAN must list tests, or state why automation is not feasible and name the fallback command.

Automated this milestone: allowlist unit tests, fixture hwmon scan, support script dry-run (must not spawn apt/modprobe), CLI `--help`/JSON smoke, grep that `set_pwm` is absent.
