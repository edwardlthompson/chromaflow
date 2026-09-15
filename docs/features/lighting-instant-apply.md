# Feature: lighting-instant-apply

> Swatch, wheel, and effect picks apply immediately; chips and All devices gauges stay square. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: swatch or wheel commit paints that color and selects Solid Color; effect `<select>` applies on change; no Lighting Apply buttons; color chips are squares; All devices Hardware gauges card stays `--picker-size` square
- ✅ Offline/error behavior: Vite still shows `lighting.applyNeedGui`; failed color apply reverts the swatch
- ✅ Accessibility: chips keep labels; effect select stays labelled; picker `aria-busy` while that row applies; Venmo is a button in Support About (not the header)
- ✅ i18n: `app.donate`; drop unused `lighting.apply` / `applyAll` / `applyAria` / `applyAllAria` / `applyEffect` / `applyEffectAria`

## Smoke scenario

1. _Given_ Lighting All devices is open
2. _When_ the user clicks a chip or releases the wheel, or picks Breathing
3. _Then_ lamps update without a second Apply click; the gauge cube is still square; Support About has Donate via Venmo

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/pages/Lighting.svelte` `lib/ColorWheel.svelte` `lib/DeviceList.svelte` `lib/EffectList.svelte` |
| View | `apps/desktop/src/app.css` `pages/Support.svelte` `locales/en.json` |
| Tests | `tests/test_chromaflow_lighting.py` `tests/test_chromaflow_tauri.py` |

## Tests

- Automated: yes — commit dispatch, Solid Color, no tools-apply, square chips, gauge-meter not stretched, Venmo URL in About
- Coverage: LED grid click still uses `applyLed`; Cooling Apply unchanged

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Never `set_pwm`. Do not put donate in the rail. PWM confirms unchanged.
