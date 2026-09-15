# Feature: desktop-tab-keepalive

> Rail tabs keep pages in memory; OpenRGB inventory is not a tab-switch tax. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling, Lighting, Profiles, and Support stay mounted; inactive panels use `.tab-hidden`; a later Lighting visit uses cached `lightInv` instead of remounting the picker
- ✅ Offline/error behavior: Vite still shows the sample fixture until a live inventory returns; Support dry-run still needs the GUI for apply
- ✅ Accessibility: hidden panels are `aria-hidden` and `inert` so rail focus does not land in the off-screen page
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ Cooling is live on this machine
2. _When_ the user opens Lighting, then Cooling, then Lighting again
3. _Then_ the second Lighting paint is the cached device list (no OpenRGB wait); Support dry-run runs on first Support visit only

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/App.svelte` `lib/lightingTick.js` `pages/Lighting.svelte` `pages/Support.svelte` |
| View | `apps/desktop/src/app.css` (`.tab-hidden`) |
| Tests | `tests/test_chromaflow_tauri.py` `tests/test_chromaflow_cooling_ui.py` `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — four `tab-hidden` panels; no `{#if tab === "Cooling"}`; Support `active`/`primed`; `hostCadence` ignores leftover modes when `devices` is `[]`; session save is debounced
- Coverage: first Lighting visit still force-loads OpenRGB; Cooling still `collect_cooling` at 400 ms

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Never `set_pwm`. Host Cycle All is not started from App.svelte. PWM confirms unchanged.
