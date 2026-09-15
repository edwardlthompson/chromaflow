# Feature: desktop-motion

> 150–200ms ease-out chrome motions, gated by `prefers-reduced-motion`. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: rail current, swatch press, toast, picker `grid-template-rows`, Watchdog pill, Ready extras, gauge marker, and Apply spinner use 120–180ms ease-out
- ✅ Offline/error behavior: `prefers-reduced-motion: reduce` stubs durations
- ✅ Accessibility: motion is decorative; focus rings stay; yellow is not text
- ✅ i18n: no copy in CSS

## Smoke scenario

1. _Given_ reduced motion is off
2. _When_ the user changes rail tab, take-over, and a gauge sample
3. _Then_ the listed transitions run; with reduced motion they do not

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/app.css` `DeviceList.svelte` |
| Tests | `tests/test_chromaflow_cooling_ui.py` `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — `transition` rules exist and `prefers-reduced-motion` still stubs duration
- Coverage: no sound, haptics, or custom cursors

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Motion only. PWM unchanged. Do not add illustrations or pull-to-refresh.
