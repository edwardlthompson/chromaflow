# Feature: desktop-apply-verb

> One primary verb: Apply. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting and Cooling primary actions say **Apply**; distinct `aria-label`s when two Apply buttons share a row
- ✅ Offline/error behavior: existing apply IPC unchanged
- ✅ Accessibility: `lighting.applyAria` `lighting.applyAllAria` `lighting.applyEffectAria` `cooling.applyAllAria`
- ✅ i18n: `lighting.apply` `lighting.applyAll` `lighting.applyEffect` `cooling.applyAll` are "Apply"

## Smoke scenario

1. _Given_ Lighting All devices and a Cooling curve card
2. _When_ the user reads the primary buttons
3. _Then_ the verb is Apply

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/locales/en.json` `DeviceList.svelte` `EffectList.svelte` `CurvesList.svelte` |
| Tests | `tests/test_chromaflow_lighting.py` `tests/test_chromaflow_cooling_ui.py` |

## Tests

- Automated: yes — locale strings; aria-labels when labels collide
- Coverage: Profiles Apply already says Apply

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Copy consistency. PWM path unchanged.
