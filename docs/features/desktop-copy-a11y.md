# Feature: desktop-copy-a11y

> First-run copy and WCAG 2.2 AA fixes without restyling Fan Control navy. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: header has no in-app ChromaFlow brand (Cinnamon title remains); Cooling toolbar drops the watchdog lecture; Support Advanced is off; competing-controller uninstall sits in a closed details; extras say Ready/Needed
- ✅ Offline/error behavior: take-over still uses `window.confirm` with rewritten `cooling.takeoverAsk`; Vite still cannot take PWM
- ✅ Accessibility: take-over checkbox `aria-label`; no `aria-description` on sliders/hex; `.fc-plus` and `.fc-hide` are 2.75rem; `prefers-reduced-motion` stub
- ✅ i18n: rewritten keys in `apps/desktop/src/locales/en.json`; `support.ready` / `support.needed`

## Smoke scenario

1. _Given_ chromaflow-gui is open
2. _When_ the user opens Cooling, Support, then a fan card
3. _Then_ the header crumb is the page name, Advanced is unchecked, and the take-over checkbox is named Take over

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/App.svelte` `pages/Cooling.svelte` `pages/Support.svelte` `ControlList.svelte` `ExtrasList.svelte` `app.css` `locales/en.json` |
| Tests | `tests/test_chromaflow_tauri.py` `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — no `aria-description`; `advanced = false`; `support.ready`; no `.brand` in App.svelte
- Coverage: takeover `aria-label`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Copy/a11y only. PWM path unchanged. gtk-rs not bumped.
