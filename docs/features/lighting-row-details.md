# Feature: lighting-row-details

> Lighting list rows lead with the lamp name; protocol/hex stays one click away. Support still shows full IDs. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: collapsed rows show name + swatch (and LED count if useful); protocol strings (`HID 0x06`, `VIA 0xA8`, I2C `0x68`) open behind a details control
- ✅ Offline/error behavior: empty protocol still omits details
- ✅ Accessibility: details control has an `aria-expanded` name from i18n; keyboard reaches it
- ✅ i18n: `lighting.rowDetails` (or similar) in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ Lighting lists Keychron / Fusion / GPU
2. _When_ the user opens a row’s details
3. _Then_ protocol/hex appears; Support extras/competitors still show full IDs

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/lib/DeviceList.svelte` |
| Tests | `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — default markup has no protocol soup on the collapsed row; details toggle exists
- Coverage: All devices stays LED-count only

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Names-first Lighting. PWM unchanged.
