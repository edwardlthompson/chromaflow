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
| FR-3 | As a user I open the GUI pages Cooling / Lighting / Profiles / Support from large rail icons | English UI; icons share rail height evenly; Support shows dry-run |
| FR-4 | As a user I am told why a device is missing | `gaps[]` uses HARDWARE.md ids; Windows-only is not faked |
| FR-5 | As a user I am warned about other fan daemons | `conflicts[]` includes fancontrol, coolercontrold, fan2go |
Open vs original brief ([`docs/PRODUCT_GAPS.md`](PRODUCT_GAPS.md)):

| ID | Story | Acceptance | Status |
|----|-------|------------|--------|
| FR-6 | As a user I launch a **standalone** Cinnamon window, not a browser | `.desktop` + `chromaflow-gui`; no URL bar; Vite preview is not product smoke | 🔲 G-APP |
| FR-7 | As a user I edit a Fan Control-like curve (display-only until daemon) | Fans/Pumps/Temps/Curves; Quiet/Balanced/Performance; mix +; take-over + calibrate | ✅ G-UX G-COOL |
| FR-8 | As a user I see OpenRGB devices, modes, and colors on localhost SDK | Device list from `127.0.0.1:6742`; color apply for listed backends; no fake Windows-only devices | ✅ G-LED |
| FR-9 | As a user I run machine-specific Install detection support | Checklist from YAML ∩ lspci/DMI; polkit `--apply` via pinned helper | 🔲 G-DET G-SUP |
| FR-10 | As a user I save a profile (curve set + RGB) | `~/.config/chromaflow/` round-trip | ✅ G-PROF |
| FR-11 | As a user curves persist across reboot with a failsafe | `chromaflowd` + systemd `ExecStopPost`; no silent 0% PWM | ✅ G-SAFE |
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
