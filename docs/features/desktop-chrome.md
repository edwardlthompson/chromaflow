# Feature: desktop-chrome

> Make the native window feel closer to Fan Control density and CoolerControl 2.0 chrome: less status prose, labeled navigation, Linux type, keyboard focus. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: header shows brand, current page, This machine/Sample pill, and Watchdog/Firmware pill (tooltips keep the long PWM copy); rail icons have visible labels; Lighting does not show `broadcast FFFFFF`; All devices drops the protocol soup; empty Uncontrolled card is omitted; Profiles is a labeled form plus a saved list, not a JSON dump
- ✅ Offline/error behavior: load errors stay `role="alert"` in the header; sample inventory still labeled Sample
- ✅ Accessibility: rail keeps `aria-label` + `aria-current`; `:focus-visible` rings on controls; page `h1` remains for AT (visually hidden when the header crumb names the page)
- ✅ i18n: `app.sourceLive`, `app.sourceSample`, `app.watchdog`, `app.firmware`, `profiles.empty`, `profiles.savedList` in `apps/desktop/src/locales/en.json` (no copy in CSS)

## Smoke scenario

1. _Given_ chromaflow-gui is open
2. _When_ the user tabs the rail and opens Lighting, then Profiles
3. _Then_ the header crumb matches the page, PWM text is a pill not a sentence, Apply success is “Color applied.”, and Profiles shows fields instead of raw JSON

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/lib/applyStatus.js` |
| View | `apps/desktop/src/App.svelte` `RailNav.svelte` `app.css` `pages/Lighting.svelte` `pages/Profiles.svelte` `DeviceList.svelte` |
| Tests | `tests/test_chromaflow_tauri.py` `tests/test_apply_status.py` |
| Wiring | App header ≤10 extra lines |

## Tests

- Automated: yes — chrome classes, rail labels, applyStatus strips `broadcast RRGGBB`, Profiles has no `JSON.stringify`
- Coverage: empty broadcast extra text still uses applyOk

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Chrome polish only. PWM apply path unchanged. gtk-rs not bumped.
