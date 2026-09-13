# Feature: lighting-hid

> USB/ARGB inventory from hidraw sysfs + liquidctl names, plus Arena 7 vendor HID apply. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting lists OpenRGB names with a hue ring + SV square, RGB/HSV sliders, suggested chips, and last two manual colors; Arena 7 and Prime Neo have Apply; hidraw uses per-VID udev
- ✅ Offline/error behavior: unknown USB is research-only (no Apply); Fusion `048d:5702` is omitted when Aorus/liquidctl already covers it; dry-run preview never apt/modprobe
- ✅ Accessibility: extra-kernel checklist on Support; color wheel and sliders are labelled
- ✅ i18n: `lighting.color` `lighting.r`–`lighting.v` `lighting.up` `lighting.down` `lighting.suggested` `lighting.recent` `lighting.extras*` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ Arena 7 hidraw is plugdev-readable and OpenRGB does not list it
2. _When_ the user Applys a color on SteelSeries Arena 7
3. _Then_ ChromaFlow writes output report `0x06` to the `0xFFC0` hidraw only, and does not write PWM

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/lighting.rs`, `arena_apply.rs`, `prime_apply.rs`, `apps/desktop/src/lib/lighting.js` |
| View | `apps/desktop/src/pages/Lighting.svelte` |
| Tests | `crates/chromaflow-core` lighting/arena tests + `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — classify known USB IDs; Arena report ID `0x06`; skip UPS
- Coverage: sysfs uevent parse, gap `usb_rgb_not_in_openrgb`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-LED Arena 7 via vendor HID (ADR-0012). Prime Neo wheel LED via hidraw 0x62 (ADR-0014). PWM remains ADR-0010.
