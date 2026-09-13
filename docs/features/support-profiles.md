# Feature: support-profiles

> Machine-specific Support dry-run **and** one-click pkexec `--apply`. Profiles stay JSON-only. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Support **Install all** runs one `pkexec` of `install-support.sh --apply` (password once; `auth_admin_keep`); per-item **Install** still uses `--only NAME`. Extra kernel support lists `it87-dkms` first, then fan/hwmon (`it87`, `nct6775`, `k10temp`, `i2c-piix4`, `jc42`, `spd5118`) and RGB I2C; `i2c-nct6775` loads `nct6775-i2c`; NVIDIA GPU I2C already loaded counts both RGB I2C extras present; extras turn green when present and show an error if that item fails; the `.deb` ships hidraw/i2c udev plus `pwm-acl` and Recommends `liquidctl` / `i2c-tools` / `lm-sensors`. Never bundle `.ko`. Profiles bind `curve_set` + `rgb`
- ✅ Offline/error behavior: repo `--apply` still refuses unless pinned helper; Vite preview cannot pkexec; illegal profile tokens rejected
- ✅ Accessibility: Support checklist is text; Install is a labelled button
- ✅ i18n: `support.install` `support.installed` `profiles.*` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ chromaflow-helper is installed at the pinned path
2. _When_ the user clicks Install detection support (experimental I2C checked)
3. _Then_ pkexec installs packages, udev, groups, and modules; no `pwm*` writes

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/lib/chromaflow_apply.py`, `crates/chromaflow-core/src/support.rs` |
| View | `apps/desktop/src/pages/Support.svelte`, `Lighting.svelte` |
| Tests | `tests/test_chromaflow_support.py`, `tests/test_chromaflow_tauri.py` |

## Tests

- Automated: yes — dry-run select, apply refuses from repo, polkit `--advanced` action, Tauri `support_apply`
- Coverage: optional apt skip; groupadd i2c/plugdev

## Fallback validation

- Command: `bash scripts/install-support.sh --dry-run --advanced` then GUI Install (pkexec)

## Definition of Done

G-SUP one-click via pinned helper. PWM remains ADR-0010.
