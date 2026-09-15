# Feature: lighting-tools-layout

> Effect menu under chips; matching squares; Cycle All does not return on tab switch. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: +/- on channel rows are centered; chips are the same 2rem rounded square as the hex preview; whites and recents sit under the suggested chips; effect `<select>` and speed sit under those; leaving Lighting does not start Cycle All
- ✅ Offline/error behavior: tab switch still works in Vite; host cycle IPC is a no-op without Tauri
- ✅ Accessibility: extra row is `lighting.moreSwatches`; Kelvin chips use `{k} K` labels
- ✅ i18n: `lighting.moreSwatches`

## Smoke scenario

1. _Given_ All devices is Solid Color
2. _When_ the user opens Cooling, then Lighting
3. _Then_ lamps stay Solid Color (host `lighting_cycle` is off); the effect menu is under the chips

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/lib/lightingTick.js` `session.js` `pages/Lighting.svelte` `App.svelte` |
| View | `apps/desktop/src/lib/ColorWheel.svelte` `app.css` |
| Tests | `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — `wantsCycle` ignores stale Cycle All keys; `--chip` size; `swatch-more`; tab save is `persistSession(name)` (debounced Rust `session_save`)
- Coverage: PWM unchanged

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Never `set_pwm`. Cycle All still runs when that effect is selected.
